export type LayoutType = 'timeline' | 'comparison' | 'process' | 'hierarchy' | 'statistical' | 'geographic' | 'circular' | 'modular' | 'narrative' | 'matrix' | 'funnel' | 'pyramid' | 'fishbone' | 'mindmap' | 'swot' | 'before-after' | 'checklist' | 'quote' | 'icon-grid' | 'flowchart' | 'dashboard';

export type StyleType = 'corporate' | 'startup' | 'minimalist' | 'playful' | 'luxury' | 'neon' | 'nature' | 'vintage' | 'flat' | 'glass' | 'gradient' | 'isometric' | 'handdrawn' | 'brutalist' | 'artdeco' | 'material' | 'swiss' | 'vaporwave' | 'monochrome' | 'duotone' | 'watercolor';

export interface InfographicItem {
  title: string;
  desc?: string;
  value?: number;
  label?: string;
  icon?: string;
}

export interface InfographicConfig {
  layout: LayoutType;
  style: StyleType;
  title: string;
  subtitle?: string;
  items: InfographicItem[];
  width?: number;
}

const STYLE_CONFIGS: Record<StyleType, {
  bg: string; title: string; text: string; accent: string; card: string; border: string; font: string;
}> = {
  corporate: { bg: '#f8f9fa', title: '#212529', text: '#495057', accent: '#0d6efd', card: '#fff', border: '#dee2e6', font: 'Inter, sans-serif' },
  startup: { bg: '#f0fdf4', title: '#0f172a', text: '#475569', accent: '#14b8a6', card: '#fff', border: '#bbf7d0', font: 'Plus Jakarta Sans, sans-serif' },
  minimalist: { bg: '#ffffff', title: '#000000', text: '#333333', accent: '#333333', card: '#f5f5f5', border: '#e0e0e0', font: 'system-ui, sans-serif' },
  playful: { bg: '#fef9ef', title: '#7c3aed', text: '#5b21b6', accent: '#f472b6', card: '#fff', border: '#fde68a', font: 'Nunito, sans-serif' },
  luxury: { bg: '#0d0d0d', title: '#d4af37', text: '#c0a060', accent: '#d4af37', card: '#1a1a1a', border: '#333333', font: 'Playfair Display, serif' },
  neon: { bg: '#0a0a23', title: '#00d4ff', text: '#c0c0c0', accent: '#00ff88', card: '#16213e', border: '#0f3460', font: 'Space Grotesk, sans-serif' },
  nature: { bg: '#f0fff0', title: '#1b4332', text: '#40916c', accent: '#2d6a4f', card: '#fff', border: '#95d5b2', font: 'Lora, serif' },
  vintage: { bg: '#fdf5e6', title: '#8b4513', text: '#a0522d', accent: '#cd853f', card: '#fff8dc', border: '#d2b48c', font: 'Crimson Pro, serif' },
  flat: { bg: '#ffffff', title: '#1e293b', text: '#475569', accent: '#3b82f6', card: '#f1f5f9', border: '#cbd5e1', font: 'Outfit, sans-serif' },
  glass: { bg: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)', title: '#fff', text: '#e2e8f0', accent: '#fbbf24', card: 'rgba(255,255,255,0.1)', border: 'rgba(255,255,255,0.2)', font: 'Sora, sans-serif' },
  gradient: { bg: 'linear-gradient(135deg, #6366f1 0%, #ec4899 100%)', title: '#fff', text: '#f8fafc', accent: '#fbbf24', card: 'rgba(255,255,255,0.15)', border: 'rgba(255,255,255,0.3)', font: 'Clash Display, sans-serif' },
  isometric: { bg: '#f8fafc', title: '#1e293b', text: '#475569', accent: '#6366f1', card: '#fff', border: '#e2e8f0', font: 'IBM Plex Sans, sans-serif' },
  handdrawn: { bg: '#fffef5', title: '#5c3d2e', text: '#7c5c4a', accent: '#e07b57', card: '#fff8f0', border: '#d4b5a0', font: 'Caveat, cursive' },
  brutalist: { bg: '#ffffff', title: '#000000', text: '#000000', accent: '#ff0000', card: '#f0f0f0', border: '#000000', font: 'Space Mono, monospace' },
  artdeco: { bg: '#1a1a2e', title: '#d4af37', text: '#c9b06b', accent: '#d4af37', card: '#16213e', border: '#d4af37', font: 'Poiret One, sans-serif' },
  material: { bg: '#fafafa', title: '#212121', text: '#757575', accent: '#6200ea', card: '#fff', border: '#e0e0e0', font: 'Roboto, sans-serif' },
  swiss: { bg: '#ffffff', title: '#000000', text: '#333333', accent: '#e30613', card: '#f5f5f5', border: '#e0e0e0', font: 'Helvetica Neue, sans-serif' },
  vaporwave: { bg: '#1a0a2e', title: '#ff71ce', text: '#c9a0ff', accent: '#01cdfe', card: '#2d1b4e', border: '#ff71ce', font: 'VCR OSD Mono, monospace' },
  monochrome: { bg: '#ffffff', title: '#000000', text: '#555555', accent: '#888888', card: '#f0f0f0', border: '#cccccc', font: 'Fira Code, monospace' },
  duotone: { bg: '#0a1628', title: '#4fc3f7', text: '#90a4ae', accent: '#4fc3f7', card: '#16213e', border: '#1a365d', font: 'Bebas Neue, sans-serif' },
  watercolor: { bg: '#fff5f5', title: '#5b2c6f', text: '#7d3c98', accent: '#af7ac5', card: '#fdebd0', border: '#d7bde2', font: 'Cormorant Garamond, serif' },
};

function renderTimeline(items: InfographicItem[], styles: typeof STYLE_CONFIGS.corporate, W: number, startY: number): string {
  let svg = '';
  const stepH = 70;
  const midX = W / 2;

  svg += `<line x1="${midX}" y1="${startY}" x2="${midX}" y2="${startY + items.length * stepH}" stroke="${styles.accent}" stroke-width="3" stroke-linecap="round"/>`;

  items.forEach((item, i) => {
    const y = startY + i * stepH;
    const side = i % 2 === 0 ? 1 : -1;
    const cx = midX + side * (W / 4);

    svg += `<circle cx="${midX}" cy="${y + 25}" r="8" fill="${styles.accent}"/>`;
    svg += `<rect x="${side > 0 ? midX + 20 : 40}" y="${y + 5}" width="${W / 2 - 40}" height="45" rx="6" fill="${styles.card}" stroke="${styles.border}" stroke-width="1"/>`;
    svg += `<text x="${cx}" y="${y + 25}" font-family="${styles.font}" font-size="13" font-weight="bold" fill="${styles.text}" text-anchor="${side > 0 ? 'start' : 'end'}">${item.title}</text>`;
    if (item.desc) {
      svg += `<text x="${cx}" y="${y + 42}" font-family="${styles.font}" font-size="11" fill="${styles.text}" text-anchor="${side > 0 ? 'start' : 'end'}">${item.desc.substring(0, 50)}</text>`;
    }
  });
  return svg;
}

function renderStatistical(items: InfographicItem[], styles: typeof STYLE_CONFIGS.corporate, W: number, startY: number): string {
  let svg = '';
  const barH = 32;
  const gap = 16;
  const maxVal = Math.max(...items.filter(i => i.value !== undefined).map(i => i.value || 0), 1);

  items.forEach((item, i) => {
    const y = startY + i * (barH + gap);
    const val = item.value || 0;
    const barW = (val / maxVal) * (W - 180);

    svg += `<text x="80" y="${y + 22}" font-family="${styles.font}" font-size="12" fill="${styles.text}" text-anchor="end">${item.label || item.title}</text>`;
    svg += `<rect x="95" y="${y}" width="${Math.max(barW, 2)}" height="${barH}" rx="4" fill="${styles.accent}" opacity="0.85"/>`;
    if (val > 0) {
      svg += `<text x="${105 + barW}" y="${y + 21}" font-family="${styles.font}" font-size="12" font-weight="bold" fill="${styles.accent}">${val}</text>`;
    }
  });
  return svg;
}

function renderProcess(items: InfographicItem[], styles: typeof STYLE_CONFIGS.corporate, W: number, startY: number): string {
  let svg = '';
  const itemW = (W - 80) / items.length;
  const y = startY + 30;

  items.forEach((item, i) => {
    const x = 40 + i * itemW;
    svg += `<circle cx="${x + itemW / 2}" cy="${y}" r="20" fill="${styles.accent}"/>`;
    svg += `<text x="${x + itemW / 2}" y="${y + 5}" text-anchor="middle" font-family="${styles.font}" font-size="14" font-weight="bold" fill="#fff">${i + 1}</text>`;
    svg += `<text x="${x + itemW / 2}" y="${y + 40}" text-anchor="middle" font-family="${styles.font}" font-size="11" fill="${styles.text}">${item.title}</text>`;
    if (i < items.length - 1) {
      svg += `<line x1="${x + itemW}" y1="${y}" x2="${x + itemW + 10}" y2="${y}" stroke="${styles.accent}" stroke-width="2" marker-end="url(#arrow)"/>`;
    }
  });
  return svg;
}

function renderComparison(items: InfographicItem[], styles: typeof STYLE_CONFIGS.corporate, W: number, startY: number): string {
  let svg = '';
  const colW = (W - 100) / 2;
  const itemH = 40;

  svg += `<rect x="40" y="${startY}" width="${colW}" height="${items.length * itemH + 10}" rx="6" fill="${styles.card}" stroke="${styles.border}"/>`;
  svg += `<rect x="${50 + colW}" y="${startY}" width="${colW}" height="${items.length * itemH + 10}" rx="6" fill="${styles.card}" stroke="${styles.border}"/>`;

  items.forEach((item, i) => {
    const y = startY + 20 + i * itemH;
    svg += `<circle cx="55" cy="${y}" r="5" fill="${styles.accent}"/>`;
    svg += `<text x="65" y="${y + 4}" font-family="${styles.font}" font-size="12" fill="${styles.text}">${item.title}</text>`;
    if (item.desc) {
      svg += `<circle cx="${60 + colW}" cy="${y}" r="5" fill="${styles.accent}" opacity="0.5"/>`;
      svg += `<text x="${70 + colW}" y="${y + 4}" font-family="${styles.font}" font-size="12" fill="${styles.text}">${item.desc.substring(0, 40)}</text>`;
    }
  });
  return svg;
}

function renderHierarchy(items: InfographicItem[], styles: typeof STYLE_CONFIGS.corporate, W: number, startY: number): string {
  let svg = '';
  const levels: string[][] = [[]];
  items.forEach((item, i) => {
    const level = Math.min(Math.floor(i / 3), 3);
    if (!levels[level]) levels[level] = [];
    levels[level].push(item.title);
  });

  levels.forEach((levelItems, i) => {
    const y = startY + i * 55;
    const levelWidth = W - 60 - i * 60;
    const startX = 30 + i * 30;
    svg += `<rect x="${startX}" y="${y}" width="${levelWidth}" height="40" rx="6" fill="${styles.accent}" opacity="${1 - i * 0.2}"/>`;
    svg += `<text x="${startX + 10}" y="${y + 25}" font-family="${styles.font}" font-size="13" font-weight="bold" fill="#fff">${levelItems.join('  |  ')}</text>`;
  });
  return svg;
}

function renderSWOT(items: InfographicItem[], styles: typeof STYLE_CONFIGS.corporate, W: number, startY: number): string {
  let svg = '';
  const qW = (W - 70) / 2;
  const qH = 150;
  const labels = ['Strengths', 'Weaknesses', 'Opportunities', 'Threats'];
  const bgColors = ['#dcfce7', '#fef3c7', '#dbeafe', '#fee2e2'];

  items.slice(0, 4).forEach((item, i) => {
    const col = i % 2;
    const row = Math.floor(i / 2);
    const x = 35 + col * (qW + 10);
    const y = startY + row * (qH + 10);

    svg += `<rect x="${x}" y="${y}" width="${qW}" height="${qH}" rx="6" fill="${bgColors[i]}" stroke="${styles.border}"/>`;
    svg += `<text x="${x + 10}" y="${y + 20}" font-family="${styles.font}" font-size="12" font-weight="bold" fill="${styles.accent}">${labels[i]}</text>`;
    if (item.desc) {
      svg += `<text x="${x + 10}" y="${y + 40}" font-family="${styles.font}" font-size="10" fill="${styles.text}">${item.desc.substring(0, 80)}</text>`;
    }
  });
  return svg;
}

function renderPyramid(items: InfographicItem[], styles: typeof STYLE_CONFIGS.corporate, W: number, startY: number): string {
  let svg = '';
  const levels = Math.min(items.length, 5);
  const levelH = 45;
  const baseW = W - 100;

  for (let i = levels - 1; i >= 0; i--) {
    const w = baseW * ((i + 1) / levels);
    const x = (W - w) / 2;
    const y = startY + (levels - 1 - i) * (levelH + 5);
    svg += `<rect x="${x}" y="${y}" width="${w}" height="${levelH}" rx="4" fill="${styles.accent}" opacity="${0.4 + (i / levels) * 0.6}"/>`;
    svg += `<text x="${W / 2}" y="${y + levelH / 2 + 5}" text-anchor="middle" font-family="${styles.font}" font-size="12" font-weight="bold" fill="#fff">${items[levels - 1 - i]?.title || ''}</text>`;
  }
  return svg;
}

function renderFunnel(items: InfographicItem[], styles: typeof STYLE_CONFIGS.corporate, W: number, startY: number): string {
  let svg = '';
  const itemH = 35;
  const maxW = W - 80;

  items.forEach((item, i) => {
    const w = maxW * (1 - i / (items.length + 1));
    const x = (W - w) / 2;
    const y = startY + i * (itemH + 8);
    svg += `<rect x="${x}" y="${y}" width="${w}" height="${itemH}" rx="4" fill="${styles.accent}" opacity="${0.3 + (1 - i / items.length) * 0.7}"/>`;
    svg += `<text x="${W / 2}" y="${y + itemH / 2 + 5}" text-anchor="middle" font-family="${styles.font}" font-size="12" font-weight="bold" fill="${i === 0 ? '#fff' : styles.text}">${item.title}</text>`;
  });
  return svg;
}

export function generateInfographic(config: InfographicConfig): string {
  const { layout, style, title, subtitle = '', items } = config;
  const styles = STYLE_CONFIGS[style];
  const W = config.width || 800;
  const H = Math.max(600, items.length * 60 + 200);

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}">`;
  svg += `<defs><marker id="arrow" markerWidth="10" markerHeight="10" refX="5" refY="5" markerUnits="userSpaceOnUse"><path d="M 0 0 L 10 5 L 0 10 Z" fill="${styles.accent}"/></marker></defs>`;

  if (styles.bg.includes('gradient')) {
    svg += `<rect width="${W}" height="${H}" fill="#0a0a23"/>`;
    svg += `<rect width="${W}" height="${H}" fill="url(#grad)"/>`;
    svg = svg.replace('</defs>', `<linearGradient id="grad" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:${styles.accent};stop-opacity:0.3"/><stop offset="100%" style="stop-color:#764ba2;stop-opacity:0.3"/></linearGradient></defs>`);
  } else {
    svg += `<rect width="${W}" height="${H}" fill="${styles.bg}"/>`;
  }

  svg += `<text x="${W / 2}" y="50" text-anchor="middle" font-family="${styles.font}" font-size="28" font-weight="bold" fill="${styles.title}">${title}</text>`;
  if (subtitle) {
    svg += `<text x="${W / 2}" y="72" text-anchor="middle" font-family="${styles.font}" font-size="14" fill="${styles.text}" opacity="0.7">${subtitle}</text>`;
  }
  svg += `<line x1="${W / 2 - 80}" y1="82" x2="${W / 2 + 80}" y2="82" stroke="${styles.accent}" stroke-width="3"/>`;

  const startY = 100;

  switch (layout) {
    case 'timeline': svg += renderTimeline(items, styles, W, startY); break;
    case 'statistical': svg += renderStatistical(items, styles, W, startY); break;
    case 'process': svg += renderProcess(items, styles, W, startY); break;
    case 'comparison': svg += renderComparison(items, styles, W, startY); break;
    case 'hierarchy': svg += renderHierarchy(items, styles, W, startY); break;
    case 'swot': svg += renderSWOT(items, styles, W, startY); break;
    case 'pyramid': svg += renderPyramid(items, styles, W, startY); break;
    case 'funnel': svg += renderFunnel(items, styles, W, startY); break;
    default: svg += renderTimeline(items, styles, W, startY);
  }

  svg += '</svg>';
  return svg;
}

export function downloadInfographic(svg: string, filename: string = 'infographic.svg') {
  const blob = new Blob([svg], { type: 'image/svg+xml' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}
