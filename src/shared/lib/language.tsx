import { useEffect, useState, type ReactNode } from "react";
import { LanguageContext, type Language } from "./language-context";

export function LanguageProvider({ children }: { children: ReactNode }) {
  const [language, setLanguage] = useState<Language>(
    () => (localStorage.getItem("nightingale-language") as Language) || "ko",
  );
  useEffect(() => {
    localStorage.setItem("nightingale-language", language);
    document.documentElement.lang = language;
  }, [language]);
  return (
    <LanguageContext.Provider value={{ language, setLanguage }}>
      {children}
    </LanguageContext.Provider>
  );
}
