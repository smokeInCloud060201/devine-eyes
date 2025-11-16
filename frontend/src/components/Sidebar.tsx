import { Link, useLocation } from 'react-router-dom';
import { LayoutDashboard, Activity, Layers, Container, Settings, HelpCircle } from 'lucide-react';
import { cn } from '../lib/utils';
import { Button } from './ui/button';

const Sidebar = () => {
  const location = useLocation();

  const navItems = [
    {
      path: '/',
      label: 'Dashboard',
      icon: LayoutDashboard,
    },
    {
      path: '/service-map',
      label: 'Service Map',
      icon: Layers,
    },
    {
      path: '/apm',
      label: 'APM',
      icon: Activity,
    },
    {
      path: '/images',
      label: 'Images',
      icon: Container,
    },
  ];

  return (
    <div className="w-64 bg-white border-r border-gray-200 shadow-sm z-10">
      {/* Logo/Branding */}
      <div className=" space-x-3 px-6 py-4 border-b border-gray-200">
        <div className="text-2xl">🐳</div>
        <span className="text-lg font-bold text-gray-900">Eyes Devine</span>
      </div>

      {/* Navigation Items */}
      <nav className="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = location.pathname === item.path;

          return (
            <Link
              key={item.path}
              to={item.path}
              className={cn(
                'flex items-center space-x-3 rounded-md px-3 py-2.5 text-sm font-medium transition-colors',
                isActive
                  ? 'bg-blue-50 text-blue-700'
                  : 'text-gray-700 hover:bg-gray-50 hover:text-gray-900'
              )}
            >
              <Icon className="h-5 w-5" />
              <span>{item.label}</span>
            </Link>
          );
        })}
      </nav>

      {/* Bottom Section */}
      <div className="border-t border-gray-200 px-3 py-4 space-y-1">
        <Button variant="ghost" className="w-full justify-start">
          <Settings className="h-5 w-5 mr-3" />
          <span>Settings</span>
        </Button>
        <Button variant="ghost" className="w-full justify-start">
          <HelpCircle className="h-5 w-5 mr-3" />
          <span>Help</span>
        </Button>
      </div>
    </div>
  );
};

export default Sidebar;

