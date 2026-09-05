import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'LLM Smart Router',
  description: 'AI 智能路由管理后台',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="zh-CN">
      <body>{children}</body>
    </html>
  );
}