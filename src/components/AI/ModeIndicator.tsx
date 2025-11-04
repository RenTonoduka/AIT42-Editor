import React from 'react';
import { Sparkles, Trophy } from 'lucide-react';

type ModeType = 'collaborative' | 'competitive';

interface ModeIndicatorProps {
  mode: ModeType;
}

export const ModeIndicator: React.FC<ModeIndicatorProps> = ({ mode }) => {
  if (mode === 'collaborative') {
    return (
      <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-purple-100 text-purple-800 text-xs font-semibold rounded-full">
        <Sparkles className="w-3.5 h-3.5" />
        アンサンブルモード
      </span>
    );
  }

  return (
    <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-yellow-100 text-yellow-800 text-xs font-semibold rounded-full">
      <Trophy className="w-3.5 h-3.5" />
      競争モード
    </span>
  );
};
