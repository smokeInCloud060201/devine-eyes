import { ReactNode } from 'react';
import Sidebar from './Sidebar';

interface PageLayoutProps {
  children: ReactNode;
}

const PageLayout = ({ children }: PageLayoutProps) => {
  return (
    <div className="min-h-screen w-full bg-gray-50 flex ">
      <Sidebar />
      <main className="flex-1 p-4">
          {children}
      </main>
    </div>
  );
};

export default PageLayout;

