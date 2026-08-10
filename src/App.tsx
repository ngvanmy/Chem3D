import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { MoleculeViewer } from './components/MoleculeViewer';
import { Settings } from './components/Settings';
import { Chat } from './components/Chat';
import { Settings as SettingsIcon, Search, History, BookOpen } from 'lucide-react';

interface CompoundData {
  id: number;
  canonical_name: string;
  common_name: string;
  formula: string;
  smiles: string;
  mw: number;
  logp: number | null;
  boiling_point: number | null;
  melting_point: number | null;
  functional_groups: string[];
  atoms: Array<{ element: string; x: number; y: number; z: number }>;
  bonds: Array<{ source: number; target: number; type: number }>;
}

function App() {
  const [query, setQuery] = useState('');
  const [compound, setCompound] = useState<CompoundData | null>(null);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [activeSuggestionIdx, setActiveSuggestionIdx] = useState(-1);
  const searchRef = useRef<HTMLDivElement>(null);

  const [history, setHistory] = useState<string[]>([]);
  const [note, setNote] = useState('');
  const [showSettings, setShowSettings] = useState(false);
  const [loading, setLoading] = useState(false);
  const [errorMsg, setErrorMsg] = useState('');

  const loadHistory = async () => {
    try {
      const hist = await invoke<string[]>('get_search_history');
      setHistory(hist);
    } catch (_) {}
  };

  useEffect(() => {
    loadHistory();
  }, []);

  // Autocomplete Suggestions Debounce Query
  useEffect(() => {
    if (!query.trim()) {
      setSuggestions([]);
      return;
    }
    const delayDebounceFn = setTimeout(async () => {
      try {
        const list = await invoke<string[]>('get_autocomplete_suggestions', { query: query.trim() });
        setSuggestions(list);
        setActiveSuggestionIdx(-1);
      } catch (err) {
        console.error(err);
      }
    }, 250);

    return () => clearTimeout(delayDebounceFn);
  }, [query]);

  // Click Outside suggestions list listener
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (searchRef.current && !searchRef.current.contains(event.target as Node)) {
        setShowSuggestions(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (showSuggestions && suggestions.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setActiveSuggestionIdx((prev) => (prev < suggestions.length - 1 ? prev + 1 : 0));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setActiveSuggestionIdx((prev) => (prev > 0 ? prev - 1 : suggestions.length - 1));
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (activeSuggestionIdx >= 0 && activeSuggestionIdx < suggestions.length) {
          const selected = suggestions[activeSuggestionIdx];
          setQuery(selected);
          setShowSuggestions(false);
          handleSearch(selected);
        } else {
          setShowSuggestions(false);
          handleSearch(query);
        }
      } else if (e.key === 'Escape') {
        setShowSuggestions(false);
      }
    } else if (e.key === 'Enter') {
      handleSearch(query);
    }
  };


  const handleSearch = async (searchQuery: string) => {
    if (!searchQuery.trim() || loading) return;
    setLoading(true);
    setErrorMsg('');
    try {
      const data = await invoke<CompoundData>('search_compound', { query: searchQuery });
      setCompound(data);
      loadHistory();
      
      // Load note for this compound
      const savedNote = await invoke<string>('get_note', { compoundId: data.id });
      setNote(savedNote);
    } catch (err: any) {
      setCompound(null);
      setErrorMsg('Không tìm thấy hợp chất: ' + err.toString());
    } finally {
      setLoading(false);
    }
  };

  const handleSaveNote = async () => {
    if (!compound) return;
    try {
      await invoke('save_note', { compoundId: compound.id, note: note });
    } catch (_) {}
  };

  const compoundContext = compound
    ? `Hợp chất: ${compound.canonical_name} (Thường gọi: ${compound.common_name}). Công thức: ${compound.formula}. SMILES: ${compound.smiles}. Nhóm chức: ${compound.functional_groups.join(', ')}.`
    : '';

  return (
    <div className="h-screen bg-slate-950 text-white flex flex-col font-sans">
      {/* Top Navbar */}
      <header className="bg-slate-900 border-b border-slate-800 px-6 py-4 flex justify-between items-center">
        <div className="flex items-center gap-2">
          <BookOpen className="text-emerald-400 w-6 h-6" />
          <span className="font-bold text-lg tracking-wide text-emerald-400">Chem3D-Tauri AI</span>
        </div>
        <button
          onClick={() => setShowSettings(!showSettings)}
          className="p-2 bg-slate-800 hover:bg-slate-700 border border-slate-700 rounded-lg transition"
        >
          <SettingsIcon className="w-5 h-5 text-slate-300" />
        </button>
      </header>

      {/* Main Workspace */}
      <main className="flex-1 flex overflow-hidden relative">
        {showSettings && (
          <div className="absolute inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
            <Settings onClose={() => setShowSettings(false)} />
          </div>
        )}

        {/* Sidebar */}
        <div className="w-64 bg-slate-900 border-r border-slate-800 flex flex-col p-4">
          <div className="relative mb-6" ref={searchRef}>
            <input
              type="text"
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setShowSuggestions(true);
              }}
              onFocus={() => setShowSuggestions(true)}
              onKeyDown={handleKeyDown}
              placeholder="Nhập tên hoặc SMILES..."
              className="w-full bg-slate-950 border border-slate-800 rounded-lg pl-9 pr-3 py-2 text-sm focus:outline-none focus:border-emerald-500 text-white placeholder-slate-500"
            />
            <Search className="absolute left-3 top-2.5 text-slate-500 w-4 h-4" />

            {showSuggestions && suggestions.length > 0 && (
              <ul className="absolute left-0 right-0 top-full mt-1 bg-slate-900/95 border border-slate-800 rounded-lg shadow-xl overflow-hidden z-50 divide-y divide-slate-800/50 backdrop-blur-md max-h-60 overflow-y-auto">
                {suggestions.map((s, i) => (
                  <li key={i}>
                    <button
                      onClick={() => {
                        setQuery(s);
                        setShowSuggestions(false);
                        handleSearch(s);
                      }}
                      className={`w-full text-left px-3 py-2 text-sm text-slate-300 hover:text-emerald-400 hover:bg-slate-800/80 transition duration-150 ${
                        i === activeSuggestionIdx ? 'bg-slate-800/90 text-emerald-400 font-medium' : ''
                      }`}
                    >
                      {s}
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>


          <div className="flex-1 overflow-y-auto">
            <div className="flex items-center gap-2 text-xs text-slate-400 font-semibold mb-3 uppercase tracking-wider">
              <History className="w-3.5 h-3.5" />
              Lịch sử tìm kiếm
            </div>
            <ul className="space-y-1.5">
              {history.map((h, i) => (
                <li key={i}>
                  <button
                    onClick={() => { setQuery(h); handleSearch(h); }}
                    className="w-full text-left text-sm text-slate-300 hover:text-emerald-400 truncate px-2 py-1 rounded hover:bg-slate-800 transition"
                  >
                    {h}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        </div>

        {/* Left/Middle Content Area */}
        <div className="flex-1 flex flex-col p-6 overflow-y-auto space-y-6">
          {errorMsg && (
            <div className="bg-red-950/40 border border-red-800 text-red-300 p-4 rounded-xl text-sm">
              {errorMsg}
            </div>
          )}

          {loading ? (
            <div className="flex-1 flex flex-col items-center justify-center">
              <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-emerald-400 mb-4"></div>
              <p className="text-slate-400">Đang tìm kiếm thông tin hóa học...</p>
            </div>
          ) : compound ? (
            <>
              {/* 3D Visualizer */}
              <div>
                <h2 className="text-xl font-bold mb-3 text-slate-200">
                  Mô hình 3D: <span className="text-emerald-400">{compound.canonical_name}</span>
                </h2>
                <MoleculeViewer atoms={compound.atoms} bonds={compound.bonds} />
              </div>

              {/* Chemical properties and notes */}
              <div className="grid grid-cols-2 gap-6">
                <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-3">
                  <h3 className="font-bold text-sm text-slate-400 uppercase tracking-wider">Thuộc tính hóa lý</h3>
                  <div className="text-sm space-y-2">
                    <div className="flex justify-between border-b border-slate-800 pb-1.5">
                      <span className="text-slate-400">Tên thường gọi:</span>
                      <span>{compound.common_name || 'N/A'}</span>
                    </div>
                    <div className="flex justify-between border-b border-slate-800 pb-1.5">
                      <span className="text-slate-400">Công thức hóa học:</span>
                      <span>{compound.formula}</span>
                    </div>
                    <div className="flex justify-between border-b border-slate-800 pb-1.5">
                      <span className="text-slate-400">Khối lượng phân tử:</span>
                      <span>{compound.mw.toFixed(2)} g/mol</span>
                    </div>
                    <div className="flex justify-between border-b border-slate-800 pb-1.5">
                      <span className="text-slate-400">Nhóm chức chính:</span>
                      <span>{compound.functional_groups.join(', ') || 'None'}</span>
                    </div>
                  </div>
                </div>

                <div className="bg-slate-900 border border-slate-800 rounded-xl p-5 flex flex-col h-full">
                  <h3 className="font-bold text-sm text-slate-400 uppercase tracking-wider mb-2">Ghi chú cá nhân</h3>
                  <textarea
                    value={note}
                    onChange={(e) => setNote(e.target.value)}
                    onBlur={handleSaveNote}
                    className="flex-1 w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-sm text-slate-300 focus:outline-none focus:border-emerald-500 resize-none min-h-[100px]"
                    placeholder="Viết ghi chú của bạn cho hợp chất này..."
                  />
                  <div className="text-right text-[10px] text-slate-500 mt-1">Tự động lưu khi rời vùng nhập</div>
                </div>
              </div>
            </>
          ) : (
            <div className="flex-1 flex flex-col items-center justify-center text-center p-8 bg-slate-900/40 border border-slate-800 rounded-2xl">
              <Search className="w-12 h-12 text-slate-600 mb-4 animate-pulse" />
              <h3 className="text-lg font-semibold text-slate-300">Chào mừng đến với Chem3D-Tauri</h3>
              <p className="text-slate-500 text-sm max-w-sm mt-2">
                Hãy nhập tên hợp chất (ví dụ: "Ethanol", "Water", "Aspirin") vào ô tìm kiếm để bắt đầu học tập trực quan.
              </p>
            </div>
          )}
        </div>

        {/* Right Chatbot Area */}
        <div className="w-96 bg-slate-900 border-l border-slate-800 p-4 flex flex-col">
          <Chat currentCompoundContext={compoundContext} />
        </div>
      </main>
    </div>
  );
}

export default App;
