interface Props {}
export function Banner({ children }: React.PropsWithChildren<Props>) {
  return (
    <div className="w-full bg-zinc-800 text-center py-3.75 px-4 text-zinc-50 text-sm">
      {children}
    </div>
  );
}
