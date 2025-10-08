import { createFileRoute } from '@tanstack/react-router';
import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';

// Components
import { TrendingUpIcon } from '~/components/icons/TrendingUpIcon';
import { Badge } from '~/components/ui/Badge';

export const Route = createFileRoute('/')({
  component: DashboardPage,
});

function DashboardPage() {
  return (
    <div className="mx-16 py-8">
      <h1 className="text-3xl/[40px] font-semibold text-[#2B2B2B]">Overview</h1>
      <p className="mt-3 text-[#42434D]">Check status and performance for all your extensions and resources.</p>

      <div className="mt-10 bg-[#F9F9F9] border-[0.5px] border-[#CBD5E1] rounded-xl p-6">
        <h2 className="text-4.25 font-semibold">Partner-chains</h2>

        <div className="grid grid-cols-1 lg:grid-cols-2 2xl:grid-cols-3 gap-7 mt-8">
          <div className="bg-white rounded-3xl p-6 shadow-[1px_0px_16px_0px_rgba(0,0,0,0.1)]">
            <div className="flex flex-row gap-3 items-center">
              <img src="/images/midnight.svg" alt="Midnight Logo" className="h-11 w-11" />
              <div className="flex-1">
                <h3 className="font-semibold text-lg text-[#2B2B2B]">Midnight Node</h3>
                <div className="text-xs text-[#969FAB] mt-1">ID 12345</div>
              </div>
              <Badge label="Connected" style="success" size="small" />
            </div>
            <div className="mt-8 font-medium text-[#585858]">
              <span className="text-[#969FAB]">Network | </span>Mainnet
            </div>
            <div className="flex items-center mt-8 py-2 px-2.5 border-l-2 border-[#404D61] text-[#404D61] bg-[#F8F8F8] rounded-sm gap-2">
              <div className="flex flex-1 items-center gap-2 text-xs">
                <div>Node operation ROI</div>
                <div className="w-[0.5px] bg-[#404D61] self-stretch" />
                <div className="text-[#2B2B2B] font-semibold">12%</div>
                <TrendingUpIcon strokeWidth={1.5} className="text-[#69C876] h-4 w-4" />
              </div>
              <InfoCircleIcon className="w-3 h-3 c" />
            </div>

            <div className="flex items-center gap-3 mt-8 text-sm flex-wrap">
              <div className="flex gap-2 px-2 py-1 rounded-sm bg-[#F8F8F8]">
                <div className="flex items-center gap-0.5 text-[#404D61]">
                  Sync <InfoCircleIcon className="w-3 h-3" />
                </div>
                <div className="w-[0.5px] bg-[#404D61] self-stretch" />
                <div className="text-[#2B2B2B] font-semibold">100%</div>
              </div>

              <div className="flex gap-2 px-2 py-1 rounded-sm bg-[#F8F8F8]">
                <div className="flex items-center gap-0.5 text-[#404D61]">
                  Uptime <InfoCircleIcon className="w-3 h-3" />
                </div>
                <div className="w-[0.5px] bg-[#404D61] self-stretch" />
                <div className="text-[#2B2B2B] font-semibold">99.97%</div>
              </div>

              <div className="flex gap-2 px-2 py-1 rounded-sm bg-[#F8F8F8]">
                <div className="flex items-center gap-0.5 text-[#404D61]">
                  Peers <InfoCircleIcon className="w-3 h-3" />
                </div>
                <div className="w-[0.5px] bg-[#404D61] self-stretch" />
                <div className="text-[#2B2B2B] font-semibold">32</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
