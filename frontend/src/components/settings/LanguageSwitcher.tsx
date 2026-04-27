import React, { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useConfigStore, CONFIG_KEYS } from '@/stores/useConfigStore'
import { cn } from '@/utils/cn'

interface LanguageSwitcherProps {
  className?: string
}

export const LanguageSwitcher: React.FC<LanguageSwitcherProps> = ({ className }) => {
  const { i18n, t } = useTranslation()
  const { language, updateField } = useConfigStore()

  // Sync i18n with persisted language on mount
  useEffect(() => {
    if (language) {
      i18n.changeLanguage(language)
    }
  }, [language, i18n])

  const switchLanguage = (lng: string) => {
    i18n.changeLanguage(lng)
    updateField(CONFIG_KEYS.LANGUAGE, lng)
  }

  const languages = [
    { code: 'zh', label: t('language.zh') },
    { code: 'en', label: t('language.en') },
  ]

  return (
    <div className={className || 'language-switcher'}>
      {languages.map((lang) => (
        <button
          key={lang.code}
          onClick={() => switchLanguage(lang.code)}
          className={cn(
            'px-3 py-1 rounded transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500',
            language === lang.code
              ? 'bg-primary-600 text-white font-medium'
              : 'bg-gray-200 text-gray-800 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600'
          )}
          aria-label={`Switch to ${lang.label}`}
        >
          {lang.label}
        </button>
      ))}
    </div>
  )
}
