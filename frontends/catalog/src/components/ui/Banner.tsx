interface Props {}
export function Banner({ children }: React.PropsWithChildren<Props>) {
  return (
    <div className="w-full bg-zinc-500 text-center py-3.75 text-white text-sm">
      {children}
    </div>
  );
}
