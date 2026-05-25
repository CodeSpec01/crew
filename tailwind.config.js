/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        'background': '#090A0F',
        'surface': 'rgba(18, 20, 29, 0.4)',
        'border': '#2A2D3D',
        'primary-container': '#00F0FF',
        'secondary': '#8A2BE2',
        'text-primary': '#FFFFFF',
        'text-secondary': '#8B949E',
        'error': '#ffb4ab',
      },
      fontFamily: {
        'sans': ['Inter', 'sans-serif'],
        'mono': ['JetBrains Mono', 'monospace'],
      }
    }
  },
  plugins: []
};