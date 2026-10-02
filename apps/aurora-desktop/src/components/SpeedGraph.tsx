import React from 'react';
import { formatSpeed } from '../utils/formatters';

interface SpeedGraphProps {
  speedHistory: number[];
}

export const SpeedGraph: React.FC<SpeedGraphProps> = ({ speedHistory }) => {
  const width = 800;
  const height = 90;
  const padding = 10;

  const maxSpeed = Math.max(...speedHistory, 1024 * 1024); // at least 1 MB/s scale
  const currentSpeed = speedHistory[speedHistory.length - 1] || 0;

  const points = speedHistory.map((val, idx) => {
    const x = padding + (idx / (speedHistory.length - 1)) * (width - 2 * padding);
    const y = height - padding - (val / maxSpeed) * (height - 2 * padding);
    return { x, y };
  });

  // Build SVG path
  let pathD = `M ${points[0].x} ${points[0].y}`;
  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[i];
    const p1 = points[i + 1];
    const mx = (p0.x + p1.x) / 2;
    pathD += ` C ${mx} ${p0.y}, ${mx} ${p1.y}, ${p1.x} ${p1.y}`;
  }

  const areaD = `${pathD} L ${points[points.length - 1].x} ${height} L ${points[0].x} ${height} Z`;

  return (
    <div className="bg-[#ffffff] border-b border-[rgba(0,0,0,0.08)] px-5 py-2 select-none shrink-0 transition-all duration-200">
      <div className="flex items-center justify-between mb-1 text-[11px] text-[#86868b]">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-[#1d1d1f]">Live Bandwidth Telemetry (60s)</span>
          <span className="text-[10px] bg-[#007aff]/10 text-[#007aff] px-1.5 py-0.2 rounded-md font-semibold">
            {formatSpeed(currentSpeed)}
          </span>
        </div>
        <span className="text-[10px]">Peak: {formatSpeed(maxSpeed)}</span>
      </div>

      <div className="relative w-full h-[70px] bg-[#fbfbfd] rounded-[8px] border border-[rgba(0,0,0,0.06)] overflow-hidden">
        <svg
          viewBox={`0 0 ${width} ${height}`}
          className="w-full h-full preserve-3d"
          preserveAspectRatio="none"
        >
          <defs>
            <linearGradient id="speedGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#007aff" stopOpacity="0.25" />
              <stop offset="100%" stopColor="#007aff" stopOpacity="0.0" />
            </linearGradient>
          </defs>

          {/* Grid lines */}
          <line
            x1="0"
            y1={height / 2}
            x2={width}
            y2={height / 2}
            stroke="rgba(0,0,0,0.04)"
            strokeDasharray="4 4"
          />

          {/* Gradient area */}
          <path d={areaD} fill="url(#speedGradient)" />

          {/* Spline line */}
          <path
            d={pathD}
            fill="none"
            stroke="#007aff"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />

          {/* Current pulse dot */}
          <circle
            cx={points[points.length - 1].x}
            cy={points[points.length - 1].y}
            r="3.5"
            fill="#007aff"
            stroke="#ffffff"
            strokeWidth="1.5"
          />
        </svg>
      </div>
    </div>
  );
};
