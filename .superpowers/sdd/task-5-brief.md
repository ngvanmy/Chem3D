### Task 5: Frontend UI/UX Completion

**Files:**
- Create: `d:\Chem3D\src\components\Settings.tsx`, `d:\Chem3D\src\components\Chat.tsx`
- Modify: `d:\Chem3D\src\App.tsx`
- Test: Run the app, try search, view structure, edit settings, query AI.

**Interfaces:**
- Consumes: `MoleculeViewer` React component, Tauri invoke commands.
- Produces: Complete learning interface.

- [ ] **Step 1: Create Settings Screen**

Create `d:\Chem3D\src\components\Settings.tsx`:
```tsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SettingsProps {
  onClose: () => void;
}

export const Settings: React.FC<SettingsProps> = ({ onClose }) => {
  const [geminiKey, setGeminiKey] = useState('');
  const [openaiKey, setOpenaiKey] = useState('');
  const [localUrl, setLocalUrl] = useState('http://localhost:11434/v1');
  const [localModel, setLocalModel] = useState('');
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [statusMsg, setStatusMsg] = useState('');

  useEffect(() => {
    const loadSettings = async () => {
      try {
        const gKey = await invoke<string>('get_setting', { key: 'gemini_api_key' });
        const oKey = await invoke<string>('get_setting', { key: 'openai_api_key' });
        const lUrl = await invoke<string>('get_setting', { key: 'local_ai_url' }) || 'http://localhost:11434/v1';
        const lModel = await invoke<string>('get_setting', { key: 'local_ai_model' });
        setGeminiKey(gKey);
        setOpenaiKey(oKey);
        setLocalUrl(lUrl);
        setLocalModel(lModel);
      } catch (err) {
        console.error(err);
      }
    };
    loadSettings();
  }, []);

  const saveSettings = async () => {
    try {
      await invoke('save_setting', { key: 'gemini_api_key', value: geminiKey });
      await invoke('save_setting', { key: 'openai_api_key', value: openaiKey });
      await invoke('save_setting', { key: 'local_ai_url', value: localUrl });
      await invoke('save_setting', { key: 'local_ai_model', value: localModel });
      setStatusMsg('Đã lưu cài đặt!');
    } catch (err: any) {
      setStatusMsg('Lỗi: ' + err.toString());
    }
  };

  const testLocalConnection = async () => {
    setStatusMsg('Đang kiểm tra kết nối...');
    try {
      const models = await invoke<string[]>('test_local_connection', { endpoint: localUrl });
      setAvailableModels(models);
      if (models.length > 0 && !localModel) {
        setLocalModel(models[0]);
      }
      setStatusMsg(`Kết nối thành công! Đã tìm thấy ${models.length} model.`);
    } catch (err: any) {
      setStatusMsg('Kết nối thất bại: ' + err.toString());
    }
  };

  return (
    <div className="bg-slate-900 border border-slate-800 p-6 rounded-2xl w-full max-w-lg mx-auto">
      <h2 className="text-xl font-bold mb-4 text-emerald-400">Cấu hình Hệ thống AI</h2>
      
      <div className="space-y-4">
        <div>
          <label className="block text-sm text-slate-400 mb-1">Google Gemini API Key</label>
          <input
            type="password"
            value={geminiKey}
            onChange={(e) => setGeminiKey(e.target.value)}
            className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-emerald-500"
            placeholder="AI Key"
          />
        </div>
        
        <div>
          <label className="block text-sm text-slate-400 mb-1">OpenAI API Key</label>
          <input
            type="password"
            value={openaiKey}
            onChange={(e) => setOpenaiKey(e.target.value)}
            className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-emerald-500"
            placeholder="sk-..."
          />
        </div>

        <div className="border-t border-slate-800 pt-4">
          <label className="block text-sm text-slate-400 mb-1">Ollama / LM Studio API Url</label>
          <div className="flex gap-2">
            <input
              type="text"
              value={localUrl}
              onChange={(e) => setLocalUrl(e.target.value)}
              className="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-emerald-500"
            />
            <button
              onClick={testLocalConnection}
              className="bg-emerald-600 hover:bg-emerald-700 px-4 py-2 rounded-lg text-sm font-semibold transition"
            >
              Test
            </button>
          </div>
        </div>

        <div>
          <label className="block text-sm text-slate-400 mb-1">Active Local Model</label>
          {availableModels.length > 0 ? (
            <select
              value={localModel}
              onChange={(e) => setLocalModel(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-white focus:outline-none"
            >
              {availableModels.map((m) => (
                <option key={m} value={m}>{m}</option>
              ))}
            </select>
          ) : (
            <input
              type="text"
              value={localModel}
              onChange={(e) => setLocalModel(e.target.value)}
              placeholder="Ví dụ: phi3, llama3"
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-emerald-500"
            />
          )}
        </div>

        <div className="flex gap-3 pt-4 justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 border border-slate-700 rounded-lg text-slate-300 hover:bg-slate-800 transition"
          >
            Đóng
          </button>
          <button
            onClick={saveSettings}
            className="px-4 py-2 bg-emerald-500 hover:bg-emerald-600 rounded-lg font-semibold transition"
          >
            Lưu
          </button>
        </div>

        {statusMsg && <div className="mt-2 text-sm text-center text-emerald-400">{statusMsg}</div>}
      </div>
    </div>
  );
};
```

- [ ] **Step 2: Create Chat Assistant Panel**

Create `d:\Chem3D\src\components\Chat.tsx`:
```tsx
import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Message {
  role: 'user' | 'assistant';
  content: string;
}

interface ChatProps {
  currentCompoundContext: string;
}

export const Chat: React.FC<ChatProps> = ({ currentCompoundContext }) => {
  const [messages, setMessages] = useState<Message[]>([
    { role: 'assistant', content: 'Xin chào! Tôi có thể giải thích gì cho bạn về các hợp chất hóa học hôm nay?' }
  ]);
  const [input, setInput] = useState('');
  const [provider, setProvider] = useState<'gemini' | 'openai' | 'local'>('local');
  const [loading, setLoading] = useState(false);

  const sendMessage = async () => {
    if (!input.trim() || loading) return;
    const userMsg = input;
    setInput('');
    setMessages((prev) => [...prev, { role: 'user', content: userMsg }]);
    setLoading(true);

    try {
      const activeModel = provider === 'openai' ? 'gpt-4o' : '';
      const reply = await invoke<string>('query_ai', {
        provider,
        prompt: userMsg,
        model: activeModel,
        context: currentCompoundContext
      });

      setMessages((prev) => [...prev, { role: 'assistant', content: reply }]);
    } catch (err: any) {
      setMessages((prev) => [...prev, { role: 'assistant', content: `Lỗi: ${err.toString()}` }]);
    } finally {
      setLoading(false);
    }
  };

  const askAboutCompound = () => {
    setInput('Giải thích cho tôi các tính chất hóa học và độ hoạt động của chất này.');
  };

  return (
    <div className="flex flex-col h-full bg-slate-950 border border-slate-800 rounded-xl overflow-hidden">
      <div className="bg-slate-900 border-b border-slate-800 p-4 flex justify-between items-center">
        <h3 className="font-bold text-emerald-400">Trợ Lý Hóa Học AI</h3>
        <select
          value={provider}
          onChange={(e: any) => setProvider(e.target.value)}
          className="bg-slate-950 border border-slate-800 text-xs px-2 py-1 rounded text-white focus:outline-none"
        >
          <option value="local">Local (Ollama)</option>
          <option value="gemini">Gemini Cloud</option>
          <option value="openai">OpenAI Cloud</option>
        </select>
      </div>

      <div className="flex-1 p-4 overflow-y-auto space-y-3">
        {messages.map((m, idx) => (
          <div key={idx} className={`flex ${m.role === 'user' ? 'justify-end' : 'justify-start'}`}>
            <div className={`max-w-[85%] px-3 py-2 rounded-lg text-sm ${m.role === 'user' ? 'bg-emerald-600 text-white' : 'bg-slate-900 text-slate-300'}`}>
              {m.content}
            </div>
          </div>
        ))}
        {loading && <div className="text-xs text-slate-500 italic">AI đang trả lời...</div>}
      </div>

      {currentCompoundContext && (
        <div className="px-4 py-2 bg-slate-900 border-t border-slate-800">
          <button
            onClick={askAboutCompound}
            className="text-xs bg-emerald-800/40 hover:bg-emerald-800/80 border border-emerald-700/60 px-3 py-1 rounded transition text-emerald-400"
          >
            Hỏi về chất hiện tại
          </button>
        </div>
      )}

      <div className="p-3 border-t border-slate-800 bg-slate-900 flex gap-2">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && sendMessage()}
          placeholder="Nhập câu hỏi..."
          className="flex-1 bg-slate-950 border border-slate-800 px-3 py-2 text-sm rounded-lg text-white focus:outline-none"
        />
        <button
          onClick={sendMessage}
          className="bg-emerald-600 hover:bg-emerald-700 px-4 py-2 rounded-lg text-sm font-semibold transition"
        >
          Gửi
        </button>
      </div>
    </div>
  );
};
```

- [ ] **Step 3: Complete App.tsx shell with sidebar and split screens**

Modify `d:\Chem3D\src\App.tsx` to build the full modern dark-mode user interface:
```tsx
import { useState, useEffect } from 'react';
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
      setErrorMsg('Không tìm thấy hợp chất: ' + err.toString());
    } finally {
      setLoading(false);
    }
  };

  const handleSaveNote = async () => {
    if (!compound) return;
    try {
      await invoke('save_note', { compoundId: compound.id, content: note });
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
          <div className="relative mb-6">
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch(query)}
              placeholder="Nhập tên hoặc SMILES..."
              className="w-full bg-slate-950 border border-slate-800 rounded-lg pl-9 pr-3 py-2 text-sm focus:outline-none focus:border-emerald-500"
            />
            <Search className="absolute left-3 top-2.5 text-slate-500 w-4 h-4" />
          </div>

          <div className="flex-1">
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
            <div className="flex-1 flex flex-col items-center justify-center text-center p-8 bg-slate-900/40 border border-slate-850 rounded-2xl">
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
```

- [ ] **Step 4: Verify complete flow**

1. Launch application:
```powershell
npm run tauri dev
```
2. Open Settings, input Local AI URL: `http://localhost:11434/v1` and hit Test.
3. Query "Ethanol" in search bar, verify the 3D model renders correctly and chemical properties are loaded.
4. Try writing a note, unfocus the box, restart the app and verify the note persists.
5. Choose Local AI, click "Hỏi về chất hiện tại" and verify query executes.

- [ ] **Step 5: Commit changes & Release build test**

1. Save commits:
```powershell
git add src
git commit -m "feat: complete modern dark premium UI/UX frontend with chat & settings"
```
2. Build production binary:
```powershell
npm run tauri build
```
Expected: Compiles to release target output successfully.
