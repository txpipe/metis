import { DiscordIcon, GithubIcon, XIcon } from '~/components/icons/social';
import { Section } from '~/components/Section';
import { button } from '~/components/ui/Button';

export function CommunitySection() {
  return (
    <Section
      title="Community"
      description="Lorem ipsum dolor sit amet consectetur nunc nunc sit velit eget sollicitudin sit posuere augue vestibulum eget turpis lobortis donec sapien integer."
      center
      className="bg-zinc-100 bg-[url(/images/footer-bg.svg)] bg-no-repeat bg-top bg-cover"
    >
      <div className="grid grid-col-3 grid-flow-col w-fit gap-4 mx-auto">
        <div className="w-full max-w-[289px] flex justify-center items-center flex-col gap-2 bg-white border border-zinc-200 px-6 py-8.5 rounded-l-4xl">
          <DiscordIcon className="size-13 text-zinc-800" />
          <h2 className="text-2xl font-semibold text-zinc-800">Discord</h2>
          <p className="text-zinc-500 text-center">
            Join our Discord community to chat with other developers and the Upstash team.
          </p>
          <a
            href="https://discord.gg/eVc6HJrYmP"
            target="_blank"
            rel="noopener noreferrer"
            className={button({ variant: 'outlined', className: 'mt-4' })}
          >
            Join
          </a>
        </div>
        <div className="w-full max-w-[289px] flex justify-center items-center flex-col gap-2 bg-white border border-zinc-200 px-6 py-8.5">
          <XIcon className="size-13 text-zinc-800" />
          <h2 className="text-2xl font-semibold text-zinc-800">X</h2>
          <p className="text-zinc-500 text-center">
            Follow us on X to stay up to date with the latest news from Upstash.
          </p>
          <a
            href="https://x.com/txpipe_tools"
            target="_blank"
            rel="noopener noreferrer"
            className={button({ variant: 'outlined', className: 'mt-4' })}
          >
            Follow
          </a>
        </div>
        <div className="w-full max-w-[289px] flex justify-center items-center flex-col gap-2 bg-white border border-zinc-200 px-6 py-8.5 rounded-r-4xl">
          <GithubIcon className="size-13 text-zinc-800" />
          <h2 className="text-2xl font-semibold text-zinc-800">X</h2>
          <p className="text-zinc-500 text-center">
            You can view all the projects we have developed as open source on our Github page.
          </p>
          <a
            href="https://github.com/txpipe"
            target="_blank"
            rel="noopener noreferrer"
            className={button({ variant: 'outlined', className: 'mt-4' })}
          >
            View
          </a>
        </div>
      </div>
    </Section>
  );
}
