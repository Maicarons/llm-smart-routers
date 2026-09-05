'use client';

import { useEffect, useState } from 'react';

interface HealthData {
  status: string;
  version: string;
  uptime_seconds: number;
  metrics: {
    total_requests: number;
    success_requests: number;
    failed_requests: number;
    total_tokens: number;
    providers: { name: string; requests: number; errors: number; tokens: number }[];
  };
}

export default function Dashboard() {
  const [health, setHealth] = useState<HealthData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch('/api/health/detail')
      .then((r) => r.json())
      .then(setHealth)
      .catch((e) => setError(e.message));
  }, []);

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center space-y-4">
          <h1 className="text-2xl font-bold text-red-400">连接失败</h1>
          <p className="text-slate-400">无法连接到后端服务: {error}</p>
          <p className="text-sm text-slate-500">请确保后端服务运行在 http://localhost:8080</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen p-8">
      <header className="mb-8">
        <h1 className="text-3xl font-bold text-white">LLM Smart Router</h1>
        <p className="text-slate-400 mt-1">AI 智能路由管理后台 v{health?.version || '...'}</p>
      </header>

      {/* 状态卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
        <div className="bg-slate-800 rounded-lg p-6 border border-slate-700">
          <div className="text-sm text-slate-400">总请求数</div>
          <div className="text-2xl font-bold text-white mt-1">{health?.metrics.total_requests.toLocaleString() || '—'}</div>
        </div>
        <div className="bg-slate-800 rounded-lg p-6 border border-slate-700">
          <div className="text-sm text-slate-400">成功请求</div>
          <div className="text-2xl font-bold text-green-400 mt-1">{health?.metrics.success_requests.toLocaleString() || '—'}</div>
        </div>
        <div className="bg-slate-800 rounded-lg p-6 border border-slate-700">
          <div className="text-sm text-slate-400">失败请求</div>
          <div className="text-2xl font-bold text-red-400 mt-1">{health?.metrics.failed_requests.toLocaleString() || '—'}</div>
        </div>
        <div className="bg-slate-800 rounded-lg p-6 border border-slate-700">
          <div className="text-sm text-slate-400">运行时间</div>
          <div className="text-2xl font-bold text-blue-400 mt-1">
            {health ? `${Math.floor(health.uptime_seconds / 3600)}h` : '—'}
          </div>
        </div>
      </div>

      {/* 提供商统计 */}
      <div className="bg-slate-800 rounded-lg border border-slate-700 p-6">
        <h2 className="text-lg font-semibold text-white mb-4">提供商统计</h2>
        {health?.metrics.providers && health.metrics.providers.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-slate-400 border-b border-slate-700">
                  <th className="text-left py-2">提供商</th>
                  <th className="text-right py-2">请求数</th>
                  <th className="text-right py-2">错误数</th>
                  <th className="text-right py-2">Token 数</th>
                  <th className="text-right py-2">错误率</th>
                </tr>
              </thead>
              <tbody>
                {health.metrics.providers.map((p) => (
                  <tr key={p.name} className="border-b border-slate-700/50">
                    <td className="py-3 text-white">{p.name}</td>
                    <td className="text-right py-3">{p.requests.toLocaleString()}</td>
                    <td className="text-right py-3 text-red-400">{p.errors.toLocaleString()}</td>
                    <td className="text-right py-3">{p.tokens.toLocaleString()}</td>
                    <td className="text-right py-3">
                      {p.requests > 0 ? `${((p.errors / p.requests) * 100).toFixed(1)}%` : '0%'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-slate-500 text-center py-8">暂无数据</p>
        )}
      </div>
    </div>
  );
}