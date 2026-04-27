export type SlideType = 'title' | 'content' | 'data' | 'quote' | 'comparison' | 'timeline' | 'section';

export interface SlideData {
  type: SlideType;
  title: string;
  subtitle?: string;
  bullets?: string[];
  visual?: string;
  quote?: string;
  author?: string;
  items?: { left: string; right: string }[];
}

export type ThemeType = 'light' | 'dark' | 'gradient';

export interface SlideDeckConfig {
  slides: SlideData[];
  theme: ThemeType;
  companyName?: string;
}

const THEMES: Record<ThemeType, {
  bg: string; bgStyle: string; text: string; accent: string; card: string; cardText: string; secondaryText: string;
}> = {
  light: { bg: '#ffffff', bgStyle: 'solid', text: '#1a1a2e', accent: '#4361ee', card: '#f8f9fa', cardText: '#1a1a2e', secondaryText: '#6c757d' },
  dark: { bg: '#1a1a2e', bgStyle: 'solid', text: '#f0f0f0', accent: '#4cc9f0', card: '#16213e', cardText: '#f0f0f0', secondaryText: '#a0aec0' },
  gradient: { bg: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)', bgStyle: 'gradient', text: '#ffffff', accent: '#ffd700', card: 'rgba(255,255,255,0.1)', cardText: '#ffffff', secondaryText: '#e2e8f0' },
};

function renderTitleSlide(slide: SlideData, theme: typeof THEMES.light, W: number, H: number): string {
  let svg = '';
  if (theme.bgStyle === 'gradient') {
    svg += `<rect width="${W}" height="${H}" fill="#667eea"/>`;
  } else {
    svg += `<rect width="${W}" height="${H}" fill="${theme.bg}"/>`;
  }
  svg += `<text x="${W / 2}" y="${H / 2 - 20}" text-anchor="middle" font-family="Inter, sans-serif" font-size="42" font-weight="bold" fill="${theme.text}">${slide.title}</text>`;
  if (slide.subtitle) {
    svg += `<text x="${W / 2}" y="${H / 2 + 30}" text-anchor="middle" font-family="Inter, sans-serif" font-size="20" fill="${theme.secondaryText}">${slide.subtitle}</text>`;
  }
  return svg;
}

function renderContentSlide(slide: SlideData, theme: typeof THEMES.light, W: number, H: number): string {
  let svg = '';
  svg += `<rect width="${W}" height="${H}" fill="${theme.bg}"/>`;

  // Accent line
  svg += `<rect x="40" y="30" width="60" height="4" rx="2" fill="${theme.accent}"/>`;

  // Title
  svg += `<text x="40" y="70" font-family="Inter, sans-serif" font-size="32" font-weight="bold" fill="${theme.accent}">${slide.title}</text>`;

  // Bullets
  if (slide.bullets && slide.bullets.length > 0) {
    slide.bullets.forEach((bullet, i) => {
      const y = 110 + i * 40;
      svg += `<circle cx="55" cy="${y - 5}" r="5" fill="${theme.accent}"/>`;
      svg += `<text x="70" y="${y}" font-family="Inter, sans-serif" font-size="18" fill="${theme.text}">${bullet}</text>`;
    });
  }

  // Visual area
  if (slide.visual) {
    svg += `<rect x="${W * 0.55}" y="100" width="${W * 0.4 - 40}" height="${H - 180}" rx="8" fill="${theme.card}"/>`;
    svg += `<text x="${W * 0.55 + (W * 0.4 - 40) / 2}" y="${H / 2}" text-anchor="middle" font-family="Inter, sans-serif" font-size="48" fill="${theme.accent}">${slide.visual}</text>`;
  }

  return svg;
}

function renderQuoteSlide(slide: SlideData, theme: typeof THEMES.light, W: number, H: number): string {
  let svg = '';
  svg += `<rect width="${W}" height="${H}" fill="${theme.bg}"/>`;
  svg += `<text x="100" y="120" font-family="serif" font-size="80" fill="${theme.accent}" opacity="0.3">"</text>`;
  if (slide.quote) {
    svg += `<text x="80" y="${H / 2}" font-family="Georgia, serif" font-size="28" font-style="italic" fill="${theme.text}" opacity="0.9">${slide.quote.substring(0, 120)}${slide.quote.length > 120 ? '...' : ''}</text>`;
  }
  if (slide.author) {
    svg += `<text x="80" y="${H - 60}" font-family="Inter, sans-serif" font-size="16" fill="${theme.accent}" font-weight="bold">— ${slide.author}</text>`;
  }
  return svg;
}

function renderComparisonSlide(slide: SlideData, theme: typeof THEMES.light, W: number, H: number): string {
  let svg = '';
  svg += `<rect width="${W}" height="${H}" fill="${theme.bg}"/>`;
  svg += `<text x="${W / 2}" y="60" text-anchor="middle" font-family="Inter, sans-serif" font-size="32" font-weight="bold" fill="${theme.accent}">${slide.title}</text>`;

  const colW = W / 2 - 40;
  const itemH = 36;
  const startY = 100;

  // Left column
  svg += `<rect x="40" y="${startY}" width="${colW}" height="${(slide.items?.length || 0) * itemH + 20}" rx="6" fill="${theme.card}"/>`;
  svg += `<text x="60" y="${startY + 25}" font-family="Inter, sans-serif" font-size="16" font-weight="bold" fill="${theme.accent}">Left</text>`;

  slide.items?.forEach((item, i) => {
    const y = startY + 50 + i * itemH;
    svg += `<circle cx="55" cy="${y}" r="4" fill="${theme.accent}"/>`;
    svg += `<text x="65" y="${y + 5}" font-family="Inter, sans-serif" font-size="14" fill="${theme.text}">${item.left}</text>`;
  });

  // Right column
  svg += `<rect x="${W / 2 + 20}" y="${startY}" width="${colW}" height="${(slide.items?.length || 0) * itemH + 20}" rx="6" fill="${theme.card}"/>`;
  svg += `<text x="${W / 2 + 40}" y="${startY + 25}" font-family="Inter, sans-serif" font-size="16" font-weight="bold" fill="${theme.secondaryText}">Right</text>`;

  slide.items?.forEach((item, i) => {
    const y = startY + 50 + i * itemH;
    svg += `<circle cx="${W / 2 + 35}" cy="${y}" r="4" fill="${theme.secondaryText}"/>`;
    svg += `<text x="${W / 2 + 45}" y="${y + 5}" font-family="Inter, sans-serif" font-size="14" fill="${theme.text}">${item.right}</text>`;
  });

  return svg;
}

function renderSectionSlide(slide: SlideData, theme: typeof THEMES.light, W: number, H: number): string {
  let svg = '';
  if (theme.bgStyle === 'gradient') {
    svg += `<rect width="${W}" height="${H}" fill="#667eea"/>`;
  } else {
    svg += `<rect width="${W}" height="${H}" fill="${theme.accent}"/>`;
  }
  svg += `<text x="${W / 2}" y="${H / 2}" text-anchor="middle" font-family="Inter, sans-serif" font-size="48" font-weight="bold" fill="#fff">${slide.title}</text>`;
  return svg;
}

function renderDataSlide(slide: SlideData, theme: typeof THEMES.light, W: number, H: number): string {
  let svg = '';
  svg += `<rect width="${W}" height="${H}" fill="${theme.bg}"/>`;
  svg += `<text x="40" y="60" font-family="Inter, sans-serif" font-size="32" font-weight="bold" fill="${theme.accent}">${slide.title}</text>`;

  if (slide.visual) {
    svg += `<text x="${W / 2}" y="${H / 2 + 40}" text-anchor="middle" font-family="Inter, sans-serif" font-size="64" font-weight="bold" fill="${theme.accent}">${slide.visual}</text>`;
  }

  if (slide.bullets && slide.bullets.length > 0) {
    slide.bullets.forEach((bullet, i) => {
      const y = 120 + i * 30;
      svg += `<text x="60" y="${y}" font-family="Inter, sans-serif" font-size="16" fill="${theme.text}">${bullet}</text>`;
    });
  }

  return svg;
}

export function generateSlideDeck(config: SlideDeckConfig): string {
  const { slides, theme: themeName } = config;
  const theme = THEMES[themeName];

  const W = 960;
  const H = 540;

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${W * slides.length}" height="${H}" viewBox="0 0 ${W * slides.length} ${H}">`;

  slides.forEach((slide, i) => {
    const offsetX = i * W;

    let slideSVG = '';
    switch (slide.type) {
      case 'title':
        slideSVG = renderTitleSlide(slide, theme, W, H);
        break;
      case 'quote':
        slideSVG = renderQuoteSlide(slide, theme, W, H);
        break;
      case 'comparison':
        slideSVG = renderComparisonSlide(slide, theme, W, H);
        break;
      case 'section':
        slideSVG = renderSectionSlide(slide, theme, W, H);
        break;
      case 'data':
        slideSVG = renderDataSlide(slide, theme, W, H);
        break;
      default:
        slideSVG = renderContentSlide(slide, theme, W, H);
    }

    svg += `<g transform="translate(${offsetX}, 0)">${slideSVG}</g>`;
  });

  svg += '</svg>';
  return svg;
}

export function downloadSlideDeck(svg: string, filename: string = 'slides.svg') {
  const blob = new Blob([svg], { type: 'image/svg+xml' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}
