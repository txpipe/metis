import { Children, Fragment } from 'react';

// Components
import { CaretRightIcon } from '~/components/icons';

export function Breadcrumb({ children }: React.PropsWithChildren) {
  const childrenArray = Children.toArray(children);
  return (
    <div className="flex flex-row gap-2 items-center text-[#64748B]">
      {childrenArray.map((child, index) => (
        <Fragment key={index}>
          {child}
          {index < childrenArray.length - 1 && (
            <CaretRightIcon className="size-4" />
          )}
        </Fragment>
      ))}
    </div>
  );
}
