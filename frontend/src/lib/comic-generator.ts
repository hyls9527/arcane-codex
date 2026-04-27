export type ArtStyle = 'manga' | 'american' | 'webtoon' | 'flat';
export type NarrativeTone = 'humorous' | 'educational' | 'dramatic' | 'conversational' | 'adventure' | 'mystery' | 'inspirational';

export interface ComicPanel {
  scene: string;
  dialogue: string;
  characters?: string[];
}

export interface ComicConfig {
  panels: ComicPanel[];
  style: ArtStyle;
  tone: NarrativeTone;
  title?: string;
}

const STYLES: Record<ArtStyle, {
  borderWidth: number;
  borderColor: string;
  borderRadius: number;
  bubbleShape: 'rect' | 'cloud' | 'burst';
  font: string;
  fontSize: number;
  bg: string;
}> = {
  manga: { borderWidth: 3, borderColor: '#333', borderRadius: 0, bubbleShape: 'rect', font: 'sans-serif', fontSize: 12, bg: '#fefefe' },
  american: { borderWidth: 4, borderColor: '#000', borderRadius: 2, bubbleShape: 'burst', font: 'Bangers, sans-serif', fontSize: 14, bg: '#1a1a2e' },
  webtoon: { borderWidth: 1, borderColor: '#ccc', borderRadius: 8, bubbleShape: 'cloud', font: 'sans-serif', fontSize: 13, bg: '#f0f4f8' },
  flat: { borderWidth: 2, borderColor: '#555', borderRadius: 8, bubbleShape: 'rect', font: 'sans-serif', fontSize: 12, bg: '#ffffff' },
};

const TONE_COLORS: Record<ArtStyle, Record<NarrativeTone, { primary: string; secondary: string; accent: string }>> = {
  manga: {
    humorous: { primary: '#FFB3BA', secondary: '#FFDFBA', accent: '#FFFFBA' },
    educational: { primary: '#BAFFC9', secondary: '#BAE1FF', accent: '#E0BBE4' },
    dramatic: { primary: '#BAE1FF', secondary: '#DDA0DD', accent: '#FFB3BA' },
    conversational: { primary: '#FFFFBA', secondary: '#BAFFC9', accent: '#FFDFBA' },
    adventure: { primary: '#FFDFBA', secondary: '#FFB3BA', accent: '#BAFFC9' },
    mystery: { primary: '#E0BBE4', secondary: '#BAE1FF', accent: '#DDA0DD' },
    inspirational: { primary: '#BAFFC9', secondary: '#FFFFBA', accent: '#BAE1FF' },
  },
  american: {
    humorous: { primary: '#FF6B6B', secondary: '#FFD93D', accent: '#6BCB77' },
    educational: { primary: '#4ECDC4', secondary: '#45B7D1', accent: '#96CEB4' },
    dramatic: { primary: '#2C3E50', secondary: '#E74C3C', accent: '#F39C12' },
    conversational: { primary: '#3498DB', secondary: '#2ECC71', accent: '#E67E22' },
    adventure: { primary: '#E67E22', secondary: '#E74C3C', accent: '#F1C40F' },
    mystery: { primary: '#34495E', secondary: '#8E44AD', accent: '#2C3E50' },
    inspirational: { primary: '#F1C40F', secondary: '#2ECC71', accent: '#3498DB' },
  },
  webtoon: {
    humorous: { primary: '#FFEAA7', secondary: '#FDCB6E', accent: '#FAB1A0' },
    educational: { primary: '#DFE6E9', secondary: '#B2BEC3', accent: '#81ECEC' },
    dramatic: { primary: '#FAB1A0', secondary: '#FF7675', accent: '#A29BFE' },
    conversational: { primary: '#81ECEC', secondary: '#74B9FF', accent: '#A29BFE' },
    adventure: { primary: '#FAB1A0', secondary: '#FFEAA7', accent: '#55EFC4' },
    mystery: { primary: '#A29BFE', secondary: '#6C5CE7', accent: '#DFE6E9' },
    inspirational: { primary: '#55EFC4', secondary: '#FFEAA7', accent: '#74B9FF' },
  },
  flat: {
    humorous: { primary: '#FDCB6E', secondary: '#FAB1A0', accent: '#55EFC4' },
    educational: { primary: '#00B894', secondary: '#00CEC9', accent: '#0984E3' },
    dramatic: { primary: '#E17055', secondary: '#D63031', accent: '#6C5CE7' },
    conversational: { primary: '#0984E3', secondary: '#00B894', accent: '#FDCB6E' },
    adventure: { primary: '#FAB1A0', secondary: '#FDCB6E', accent: '#00B894' },
    mystery: { primary: '#6C5CE7', secondary: '#A29BFE', accent: '#2D3436' },
    inspirational: { primary: '#00B894', secondary: '#55EFC4', accent: '#FFEAA7' },
  },
};

type PanelLayout = [number, number, number, number];

const LAYOUTS: PanelLayout[][] = [
  [[0, 0, 0.5, 1], [0.5, 0, 0.5, 1]],
  [[0, 0, 1, 0.5], [0, 0.5, 0.5, 0.5], [0.5, 0.5, 0.5, 0.5]],
  [[0, 0, 1, 1]],
  [[0, 0, 0.33, 1], [0.33, 0, 0.33, 1], [0.66, 0, 0.33, 1]],
  [[0, 0, 1, 0.6], [0, 0.6, 0.5, 0.4], [0.5, 0.6, 0.5, 0.4]],
];

function getLayout(panelCount: number): [number, number, number, number][] {
  if (panelCount <= 1) return LAYOUTS[2];
  if (panelCount === 2) return LAYOUTS[0];
  if (panelCount === 3) return LAYOUTS[1];
  if (panelCount === 4) return LAYOUTS[3];
  return LAYOUTS[4];
}

function drawSpeechBubble(
  x: number, y: number, w: number, h: number,
  text: string, shape: 'rect' | 'cloud' | 'burst',
  borderColor: string, bgColor: string = '#ffffff'
): string {
  let svg = '';
  if (shape === 'cloud') {
    const r = 12;
    svg += `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" fill="${bgColor}" stroke="${borderColor}" stroke-width="1.5"/>`;
    svg += `<polygon points="${x},${y + h} ${x + 10},${y + h + 8} ${x + 20},${y + h}" fill="${bgColor}" stroke="${borderColor}" stroke-width="1.5"/>`;
  } else if (shape === 'burst') {
    const points: string[] = [];
    const cx = x + w / 2, cy = y + h / 2;
    for (let i = 0; i < 12; i++) {
      const angle = (i * Math.PI * 2) / 12;
      const r = i % 2 === 0 ? Math.max(w, h) / 1.8 : Math.max(w, h) / 2.5;
      points.push(`${cx + Math.cos(angle) * r},${cy + Math.sin(angle) * r}`);
    }
    svg += `<polygon points="${points.join(' ')}" fill="${bgColor}" stroke="${borderColor}" stroke-width="2"/>`;
  } else {
    svg += `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="8" fill="${bgColor}" stroke="${borderColor}" stroke-width="1.5"/>`;
    svg += `<polygon points="${x},${y + h} ${x + 12},${y + h + 6} ${x + 24},${y + h}" fill="${bgColor}" stroke="${borderColor}" stroke-width="1.5"/>`;
  }
  svg += `<text x="${x + 8}" y="${y + h / 2 + 5}" font-family="sans-serif" font-size="11" fill="#333">${text.substring(0, 40)}${text.length > 40 ? '...' : ''}</text>`;
  return svg;
}

function drawCharacter(cx: number, cy: number, r: number, color: string, style: ArtStyle): string {
  let svg = '';
  svg += `<circle cx="${cx}" cy="${cy}" r="${r}" fill="${color}" opacity="0.6"/>`;
  if (style === 'manga') {
    svg += `<ellipse cx="${cx - r * 0.3}" cy="${cy - r * 0.2}" rx="${r * 0.15}" ry="${r * 0.3}" fill="#333"/>`;
    svg += `<ellipse cx="${cx + r * 0.3}" cy="${cy - r * 0.2}" rx="${r * 0.15}" ry="${r * 0.3}" fill="#333"/>`;
  } else if (style === 'flat') {
    svg += `<rect x="${cx - r * 0.4}" y="${cy - r * 0.3}" width="${r * 0.25}" height="${r * 0.25}" rx="2" fill="#333"/>`;
    svg += `<rect x="${cx + r * 0.15}" y="${cy - r * 0.3}" width="${r * 0.25}" height="${r * 0.25}" rx="2" fill="#333"/>`;
  }
  return svg;
}

function drawSceneBg(x: number, y: number, w: number, h: number, color: string, panelIndex: number): string {
  let svg = `<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${color}" opacity="0.15"/>`;
  if (panelIndex % 3 === 0) {
    svg += `<rect x="${x + 10}" y="${y + 10}" width="${w * 0.4}" height="${h * 0.3}" rx="4" fill="${color}" opacity="0.1"/>`;
  } else if (panelIndex % 3 === 1) {
    svg += `<circle cx="${x + w * 0.7}" cy="${y + h * 0.3}" r="${Math.min(w, h) * 0.15}" fill="${color}" opacity="0.1"/>`;
  }
  return svg;
}

export function generateComicSVG(config: ComicConfig): string {
  const { panels, style, tone } = config;
  const styleConfig = STYLES[style];
  const colors = TONE_COLORS[style]?.[tone] ?? TONE_COLORS[style].humorous;
  const layout = getLayout(panels.length);

  const panelW = 360;
  const panelH = 240;
  const gap = 12;

  const cols = layout.filter(p => p[0] === 0).length > 1 ? 2 : layout[0][2] < 1 ? 2 : 1;
  const rows = Math.ceil(panels.length / cols);

  const W = cols * panelW + (cols - 1) * gap + 40;
  const H = rows * panelH + (rows - 1) * gap + 60;

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">`;
  svg += `<defs>
    <filter id="shadow" x="-2%" y="-2%" width="104%" height="104%">
      <feDropShadow dx="2" dy="2" stdDeviation="3" flood-opacity="0.15"/>
    </filter>
  </defs>`;
  svg += `<rect width="${W}" height="${H}" fill="${styleConfig.bg}"/>`;

  if (config.title) {
    svg += `<text x="${W / 2}" y="35" text-anchor="middle" font-family="${styleConfig.font}" font-size="22" font-weight="bold" fill="${colors.accent}">${config.title}</text>`;
    svg += `<line x1="${W / 2 - 100}" y1="42" x2="${W / 2 + 100}" y2="42" stroke="${colors.accent}" stroke-width="2"/>`;
  }

  const startY = config.title ? 52 : 20;

  panels.forEach((panel, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    const px = 20 + col * (panelW + gap);
    const py = startY + row * (panelH + gap);

    svg += drawSceneBg(px, py, panelW, panelH, colors.primary, i);

    svg += `<rect x="${px}" y="${py}" width="${panelW}" height="${panelH}" rx="${styleConfig.borderRadius}" fill="white" stroke="${styleConfig.borderColor}" stroke-width="${styleConfig.borderWidth}" filter="url(#shadow)"/>`;

    svg += `<text x="${px + 10}" y="${py + 18}" font-family="sans-serif" font-size="10" font-weight="bold" fill="#999">${i + 1}</text>`;

    svg += drawCharacter(px + panelW * 0.35, py + panelH * 0.45, 35, colors.primary, style);

    const bubbleX = px + panelW * 0.55;
    const bubbleY = py + 15;
    const bubbleW = panelW * 0.4;
    const bubbleH = 40;
    svg += drawSpeechBubble(bubbleX, bubbleY, bubbleW, bubbleH, panel.dialogue, styleConfig.bubbleShape, styleConfig.borderColor);

    if (panel.scene) {
      svg += `<text x="${px + 10}" y="${py + panelH - 10}" font-family="sans-serif" font-size="9" fill="#aaa" font-style="italic">[${panel.scene.substring(0, 30)}]</text>`;
    }
  });

  svg += '</svg>';
  return svg;
}

export function downloadComicSVG(svg: string, filename: string = 'comic.svg') {
  const blob = new Blob([svg], { type: 'image/svg+xml' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}
