import { NavLink, Outlet } from "react-router-dom";

const navigation = [
  { to: "/", label: "대시보드" },
  { to: "/system", label: "시스템 상태" },
  { to: "/monitoring", label: "파일 모니터링" },
  { to: "/events", label: "보안 이벤트" },
  { to: "/policy", label: "보안 정책" },
  { to: "/about", label: "정보" },
];

export function AppLayout() {
  return (
    <div className="app-shell">
      <aside>
        <div className="brand">
          Nightingale<span>보안 모니터</span>
        </div>
        <nav>
          {navigation.map(({ to, label }) => (
            <NavLink key={to} to={to} end={to === "/"}>
              {label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main>
        <Outlet />
      </main>
    </div>
  );
}
