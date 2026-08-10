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
        setGeminiKey(gKey || '');
        setOpenaiKey(oKey || '');
        setLocalUrl(lUrl || 'http://localhost:11434/v1');
        setLocalModel(lModel || '');
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
