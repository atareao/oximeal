import { lazy, Suspense } from 'react';
import { Link, Route, Routes } from 'react-router-dom';
import { Result, Skeleton } from 'antd';
import { AppLayout } from './components/AppLayout';

const MenuPage = lazy(() =>
  import('./pages/MenuPage').then((module) => ({ default: module.MenuPage })),
);
const ShoppingPage = lazy(() =>
  import('./pages/ShoppingPage').then((module) => ({ default: module.ShoppingPage })),
);

function PageFallback() {
  return (
    <div className="page-fallback" data-testid="page-fallback">
      <Skeleton active paragraph={{ rows: 6 }} />
    </div>
  );
}

export function App() {
  return (
    <Suspense fallback={<PageFallback />}>
      <Routes>
        <Route element={<AppLayout />}>
          <Route index element={<MenuPage />} />
          <Route path="shopping" element={<ShoppingPage />} />
          <Route
            path="*"
            element={
              <Result
                status="404"
                title="404"
                subTitle="La página que buscas no existe."
                extra={<Link to="/">Volver al menú</Link>}
              />
            }
          />
        </Route>
      </Routes>
    </Suspense>
  );
}
