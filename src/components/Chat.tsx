import React, { useState, useEffect } from 'react';
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
  const [localModel, setLocalModel] = useState('');
  const [openaiModel, setOpenaiModel] = useState('');

  useEffect(() => {
    const loadSavedModels = async () => {
      try {
        const lModel = await invoke<string>('get_setting', { key: 'local_ai_model' });
        const oModel = await invoke<string>('get_setting', { key: 'openai_model' });
        if (lModel) setLocalModel(lModel);
        if (oModel) setOpenaiModel(oModel);
      } catch (err) {
        console.error(err);
      }
    };
    loadSavedModels();
  }, []);

  const sendMessage = async () => {
    if (!input.trim() || loading) return;
    const userMsg = input;
    setInput('');
    setMessages((prev) => [...prev, { role: 'user', content: userMsg }]);
    setLoading(true);

    try {
      let activeModel = '';
      if (provider === 'local') {
        activeModel = localModel;
      } else if (provider === 'openai') {
        activeModel = openaiModel;
      }

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
