import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ReactMarkdown from 'react-markdown';

interface Message {
  role: 'user' | 'assistant';
  content: string;
}

interface ChatProps {
  currentCompoundContext: string;
}

const MarkdownComponents = {
  p: ({ children }: any) => <p className="mb-1.5 last:mb-0 leading-relaxed">{children}</p>,
  h1: ({ children }: any) => <h1 className="text-base font-bold my-1.5 text-emerald-400">{children}</h1>,
  h2: ({ children }: any) => <h2 className="text-sm font-bold my-1.5 text-emerald-400">{children}</h2>,
  h3: ({ children }: any) => <h3 className="text-xs font-bold my-1.5 text-emerald-400">{children}</h3>,
  ul: ({ children }: any) => <ul className="list-disc pl-4 mb-1.5 space-y-1 text-xs md:text-sm text-slate-350">{children}</ul>,
  ol: ({ children }: any) => <ol className="list-decimal pl-4 mb-1.5 space-y-1 text-xs md:text-sm text-slate-350">{children}</ol>,
  li: ({ children }: any) => <li>{children}</li>,
  strong: ({ children }: any) => <strong className="font-semibold text-emerald-300">{children}</strong>,
  code: ({ children }: any) => {
    const isInline = !String(children).includes('\n');
    return isInline ? (
      <code className="bg-slate-950 px-1 py-0.5 rounded text-[11px] font-mono text-emerald-400 border border-slate-850">
        {children}
      </code>
    ) : (
      <code className="block text-emerald-300 font-mono text-xs overflow-x-auto whitespace-pre">
        {children}
      </code>
    );
  },
  pre: ({ children }: any) => (
    <pre className="bg-slate-950 p-2 rounded-lg overflow-x-auto my-1.5 border border-slate-850 w-full max-w-full">
      {children}
    </pre>
  ),
  a: ({ href, children }: any) => (
    <a href={href} target="_blank" rel="noopener noreferrer" className="text-emerald-400 underline hover:text-emerald-300">
      {children}
    </a>
  ),
};

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
      const reply = await invoke<string>('query_ai', {
        provider,
        prompt: userMsg,
        model: '', // Empty model allows backend to query the latest settings dynamically
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
    <div className="flex flex-col h-full bg-slate-950 border border-slate-800 rounded-xl overflow-hidden font-sans">
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
            <div className={`max-w-[85%] px-3 py-2 rounded-lg text-sm ${m.role === 'user' ? 'bg-emerald-600 text-white' : 'bg-slate-900 text-slate-350 overflow-x-auto w-full'}`}>
              {m.role === 'user' ? (
                m.content
              ) : (
                <ReactMarkdown components={MarkdownComponents}>
                  {m.content}
                </ReactMarkdown>
              )}
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
