import clsx from 'clsx';
import React, { useEffect } from 'react';

interface Props {
  onTabChanged?(index: number, value?: string): void;
  initialTab?: number;
  className?: string;
}

export function Tabs({ children, onTabChanged, initialTab, className }: React.PropsWithChildren<Props>) {
  const childrenArray = React.Children.toArray(children);

  const [activeIndex, setActiveIndex] = React.useState(initialTab ?? 0);

  useEffect(() => {
    const child = childrenArray[activeIndex] as React.ReactElement<TabItemProps>;
    onTabChanged?.(activeIndex, child?.props.value);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeIndex]);

  return (
    <div className={clsx('flex flex-row border-b border-zinc-200', className)}>
      {childrenArray.map((child, index) => {
        if (React.isValidElement<TabItemProps>(child)) {
          return (
            <button
              type="button"
              data-active={index === activeIndex}
              key={index}
              className="text-zinc-500 data-[active=true]:text-[#0000FF]"
              onClick={() => setActiveIndex(index)}
            >
              {child}
            </button>
          );
        }
        return null;
      })}
    </div>
  );
}

interface TabItemProps {
  icon?: React.ReactNode;
  label: string;
  value?: string;
}

export function TabItem({ icon, label }: TabItemProps) {
  return (
    <div className="flex flex-row gap-2 px-3 py-2.5 cursor-pointer">
      {icon}
      <span>{label}</span>
    </div>
  );
}
