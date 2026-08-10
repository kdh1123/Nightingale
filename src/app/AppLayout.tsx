import { NavLink, Outlet } from "react-router-dom";
import { LanguageProvider } from "../shared/lib/language";
import { useLanguage } from "../shared/lib/use-language";

const navigation = [
  { to: "/", labelKo: "대시보드", labelEn: "Dashboard", icon: "◈" },
  { to: "/system", labelKo: "시스템", labelEn: "System", icon: "▦" },
  { to: "/monitoring", labelKo: "모니터링", labelEn: "Monitoring", icon: "◉" },
  { to: "/events", labelKo: "위협 탐지", labelEn: "Threats", icon: "◇" },
  { to: "/policy", labelKo: "리포트 및 설정", labelEn: "Reports & Settings", icon: "▤" },
];

export function AppLayout() {
  return (
    <LanguageProvider>
      <AppFrame />
    </LanguageProvider>
  );
}
function AppFrame() {
  const { language, setLanguage } = useLanguage();
  const ko = language === "ko";
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">N</span>
          <div>
            Nightingale<small>{ko ? "엔드포인트 보안" : "Endpoint Security"}</small>
          </div>
        </div>
        <nav aria-label={ko ? "주 메뉴" : "Main menu"}>
          {navigation.map(({ to, labelKo, labelEn, icon }) => (
            <NavLink key={to} to={to} end={to === "/"}>
              <span aria-hidden="true">{icon}</span>
              {ko ? labelKo : labelEn}
            </NavLink>
          ))}
        </nav>
        <div className="sidebar-foot">
          <NavLink to="/about">
            <span>◌</span>
            {ko ? "제품 정보" : "About"}
          </NavLink>
          <p>
            <i /> {ko ? "보호 기능 활성화" : "Protection active"}
          </p>
        </div>
      </aside>
      <main className="workspace">
        <div className="top-utility">
          <span>{ko ? "로컬 우선 보안" : "Local-first security"}</span>
          <div
            className="language-toggle"
            role="group"
            aria-label={ko ? "언어 선택" : "Language selection"}
          >
            <button className={ko ? "active" : ""} onClick={() => setLanguage("ko")}>
              한국어
            </button>
            <button className={!ko ? "active" : ""} onClick={() => setLanguage("en")}>
              EN
            </button>
          </div>
        </div>
        <Outlet />
      </main>
      <footer className="status-bar">
        <span>
          <i /> {ko ? "모니터링 활성화" : "Monitoring active"}
        </span>
        <span>{ko ? "데이터베이스 연결됨" : "Database connected"}</span>
        <span className="status-sync">{ko ? "마지막 동기화: 로컬" : "Last sync: local"}</span>
        <span>v0.1.0</span>
      </footer>
    </div>
  );
}
