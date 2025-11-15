import clsx from 'clsx';

// Components
import { DiscordIcon, GithubIcon, XIcon } from '~/components/icons/social';
import { Section } from '~/components/Section';
import { button } from '~/components/ui/Button';

interface CommunityCardProps {
  className?: string;
  icon: React.ReactNode;
  title: string;
  description: string;
  link: {
    href: string;
    label: string;
  };
}

function CommunityCard({ icon, title, description, link, className }: CommunityCardProps) {
  return (
    <div className={clsx('w-full max-w-[289px] flex justify-start items-center flex-col gap-2 bg-white border border-zinc-200 px-6 py-8.5 rounded-xl md:rounded-none', className)}>
      {icon}
      <h2 className="text-2xl font-semibold text-zinc-800">{title}</h2>
      <p className="text-zinc-500 text-center grow">
        {description}
      </p>
      <a
        href={link.href}
        target="_blank"
        rel="noopener noreferrer"
        className={button({ variant: 'outlined', className: 'mt-4' })}
      >
        {link.label}
      </a>
    </div>
  );
}

export function CommunitySection() {
  return (
    <Section
      title="Community"
      description="Lorem ipsum dolor sit amet consectetur nunc nunc sit velit eget sollicitudin sit posuere augue vestibulum eget turpis lobortis donec sapien integer."
      center
      className="bg-zinc-100 bg-[url(/images/footer-bg.svg)] bg-no-repeat bg-top bg-cover"
    >
      <div className="grid grid-col-3 md:grid-flow-col w-fit gap-4 mx-auto">
        <CommunityCard
          className="md:rounded-l-4xl"
          icon={<DiscordIcon className="size-13 text-zinc-800" />}
          title="Discord"
          description="Join our Discord community to chat with other developers and the Upstash team."
          link={{ href: 'https://discord.gg/eVc6HJrYmP', label: 'Join' }}
        />
        <CommunityCard
          icon={<XIcon className="size-13 text-zinc-800" />}
          title="X"
          description="Follow us on X to stay up to date with the latest news from Upstash."
          link={{ href: 'https://x.com/txpipe_tools', label: 'Follow' }}
        />
        <CommunityCard
          className="md:rounded-r-4xl"
          icon={<GithubIcon className="size-13 text-zinc-800" />}
          title="Github"
          description="You can view all the projects we have developed as open source on our Github page."
          link={{ href: 'https://github.com/txpipe', label: 'View' }}
        />
      </div>
    </Section>
  );
}
