import React from 'react'
import { useTranslation } from 'react-i18next'

interface LanguageSwitcherProps {
  className?: string
}

export const LanguageSwitcher: React.FC<LanguageSwitcherProps> = ({ className }) => {
  const { i18n } = useTranslation()
  const currentLang = i18n.language

  const switchLanguage = (lng: string) => {
    i18n.changeLanguage(lng)
  }

  const languages = [
    { code: 'zh', label: '中文' },
    { code: 'en', label: 'English' },
  ]

  return (
    <div className={className || 'language-switcher'}>
      {languages.map((lang) => (
        <button
          key={lang.code}
          onClick={() => switchLanguage(lang.code)}
          className={currentLang === lang.code ? 'active' : ''}
          aria-label={`Switch to ${lang.label}`}
        >
          {lang.label}
        </button>
      ))}
    </div>
  )
}
