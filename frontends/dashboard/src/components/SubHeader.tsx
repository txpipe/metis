import { Badge } from '~/components/ui/Badge';

interface Props {
  logo?: string;
  title: string;
  subtitle: string;
  tags?: string[];
}

export function SubHeader({ logo, title, subtitle, tags }: Props) {
  return (
    <div className="px-14 py-6 flex flex-row gap-4 items-start border-b border-neutral-200">
      {logo && <img src={logo} alt={`Logo for ${title}`} className="aspect-square w-15.5" />}
      <div>
        <h1 className="text-3xl font-semibold text-[#131316]">{title}</h1>
        <p className="text-[#42434D] text-sm">{subtitle}</p>
        {tags && (
          <div className="flex flex-row gap-2 mt-2">
            {tags.map(tag => (
              <Badge key={tag} label={tag} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
