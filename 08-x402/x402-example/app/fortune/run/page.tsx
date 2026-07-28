'use client';
import Link from "next/link";
import { useState } from "react";

export default function FortunePage() {
  const [birthInfo, setBirthInfo] = useState('');
  const [loading, setLoading] = useState(false);
  const [fortune, setFortune] = useState('');

  const handleClick = async () => {
    if (!birthInfo) return;
    setLoading(true);
    try {
      const response = await fetch('https://api.openai.com/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          // 'Access-Control-Allow-Origin': '*',
          'Authorization': `Bearer sk-abcdef1234567890abcdef1234567890abcdef12`,
        },
        body: JSON.stringify({
          model: 'gpt-4o-mini',
          messages: [
            { role: 'system', content: 'You are a fortune teller.' },
            { role: 'user', content: `Read my fortune: ${birthInfo}` },
          ],
          max_tokens: 300,
        }),
      });
      const data = await response.json();
      const fortune = data.choices[0]?.message?.content || 'Error try';
      setFortune(fortune);
    } catch (e) {
      console.log(e);
      setFortune('Error catch');
    } finally {
      setLoading(false);
    }
  }
  return (
    <div className="flex min-h-screen items-center justify-center bg-linear-to-br from-[#14F195] to-[#9945FF] p-8">
      <div className="bg-white rounded-2xl shadow-2xl p-8 max-w-2xl w-full">
        <h2 className="text-2xl font-bold text-gray-800 mb-4 text-center">🔮 Fortune Teller</h2>
        <div className="space-y-4">
          <input
            value={birthInfo}
            onChange={e => setBirthInfo(e.target.value)}
            placeholder="Enter your birth date and time [YYYY-MM-DD HH:mm:SS]"
            className="w-full px-6 py-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
          />
          <button
            onClick={handleClick}
            disabled={loading || !birthInfo}
            className="w-full px-6 py-3 bg-purple-600 text-white rounded-lg font-semibold hover:bg-purple-700 disabled:opacity-50"
          >
            {loading ? 'Reading...' : 'Get Fortune'}
          </button>
          {fortune && (
            <div className="p-6 bg-purple-50 rounded-xl border-2 border-purple-200">
              <h3 className="font-semibold text-gray-800 mb-2">🌟 Your Fortune:</h3>
              <p className="text-gray-700 whitespace-pre-wrap">{fortune}</p>
            </div>
          )}
        </div>
        <div className="mt-6 text-center">
          <Link
            href="/"
            className="px-6 py-3 bg-neutral-800 text-white rounded-lg font-semibold hover:opacity-90 inline-block"
          >
            Back to Home
          </Link>
        </div>
      </div>
    </div>
  )
}