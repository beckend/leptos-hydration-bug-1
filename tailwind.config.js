/** @type {import('tailwindcss').Config} */
module.exports = {
  daisyui: {
    darkTheme: 'dark',
    logs: false,
    themes: [
      {
        dark: {
          primary: '#0063db',
          secondary: '#a6ef75',
          accent: '#2ecc40',
          neutral: '#292a38',
          'base-100': '#111827',
          info: '#8fdaf5',
          success: '#19d25d',
          warning: '#f4cb62',
          error: '#ef484b',
        },
        light: {
          primary: '#2ecc40',
          secondary: '#0063db',
          accent: '#6ffc7a',
          neutral: '#1f202d',
          'base-100': '#faf9fb',
          info: '#80b6e5',
          success: '#28dc88',
          warning: '#d1ad10',
          error: '#f1786f',
        },
      },
    ],
  },
  content: ['*.html', './src/**/*.rs',],
  plugins: [require('@tailwindcss/typography'), require('daisyui')],
  darkMode: 'class',
  theme: {
    fontFamily: {
      sans: ['Montserrat'],
    },
  },
}

