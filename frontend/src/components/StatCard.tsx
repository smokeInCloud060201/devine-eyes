import { Card, CardContent } from './ui/card';

interface StatCardProps {
  title: string;
  value: string;
  unit?: string;
}

const StatCard = ({ title, value, unit }: StatCardProps) => {
  return (
    <Card className="transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md">
      <CardContent className="p-5">
        <div className="text-sm text-gray-600 mb-2 font-medium">{title}</div>
        <div className="text-3xl font-bold text-gray-900 flex items-baseline gap-1">
          {value}
          {unit && <span className="text-base font-normal text-gray-600">{unit}</span>}
        </div>
      </CardContent>
    </Card>
  );
};

export default StatCard;

