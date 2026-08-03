import { createContext } from "react";

export type Language = "ko" | "en";
export const LanguageContext = createContext<{
  language: Language;
  setLanguage: (language: Language) => void;
}>({ language: "ko", setLanguage: () => undefined });
