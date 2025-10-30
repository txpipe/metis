import { createFileRoute, Link, redirect } from '@tanstack/react-router';
import clsx from 'clsx';
import { twMerge } from 'tailwind-merge';

// Components
import { CaretRightIcon } from '~/components/icons/CaretRightIcon';
import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';
import { Card } from '~/components/Card';

// Data
import { workloads } from '~/data/workloads';

export const Route = createFileRoute('/$workloadId')({
  loader: async ({ params: { workloadId } }) => {
    const workload = workloads.find(w => w.id === workloadId);
    if (!workload) {
      throw redirect({
        to: '/',
      });
    }

    workload.rewards = '300 Nights';
    workload.uptime = 100;
    workload.status = 'connected';

    return { workload };
  },
  component: WorkloadIdInfo,
});

function InfoChip({ label, value, valueClassName }: { label: string; value: string; valueClassName?: string; }) {
  return (
    <div className="flex flex-row items-center gap-2 py-1 px-3 rounded-full border-[0.5px] border-[#969FAB]">
      <div className="flex flex-row gap-1 items-center text-[#969FAB] text-sm font-medium">
        {label}
        <InfoCircleIcon className="w-3 h-3" />
      </div>
      <div className="w-px self-stretch bg-[#969FAB]" />
      <div className={twMerge('text-sm text-[#2B2B2B]/80', valueClassName)}>{value}</div>
    </div>
  );
}

const dummyLogs = `2025-02-10 10:00:01 INFO [Server] - Server started successfully.
2025-02-10 10:00:02 INFO [Database] - Connection to database established.
2025-02-10 10:00:03 INFO [Auth] - User login attempt: user_id=12345
2025-02-10 10:00:04 WARN [Auth] - Failed login attempt for user_id=12345 (Invalid password).
2025-02-10 10:00:05 INFO [Auth] - User login successful: user_id=67890
2025-02-10 10:00:06 INFO [API] - Request received: GET /api/products
2025-02-10 10:00:07 DEBUG [DBQuery] - Executing query: SELECT * FROM products WHERE available = 1
2025-02-10 10:00:08 INFO [API] - Response sent: 200 OK (15ms)
2025-02-10 10:00:09 INFO [Payment] - Payment process started: transaction_id=ABC123
2025-02-10 10:00:10 ERROR [Payment] - Payment failed: transaction_id=ABC123, reason=Insufficient funds
2025-02-10 10:00:11 INFO [User] - User profile updated: user_id=67890
2025-02-10 10:00:12 DEBUG [Cache] - Cache hit for key: user_67890_profile
2025-02-10 10:00:13 WARN [Security] - Unauthorized access attempt detected from IP: 192.168.1.100
2025-02-10 10:00:14 INFO [Email] - Password reset email sent to user_id=12345
2025-02-10 10:00:15 DEBUG [Queue] - Job enqueued: job_id=XYZ987, type=email
2025-02-10 10:00:16 INFO [API] - Request received: POST /api/orders
2025-02-10 10:00:17 DEBUG [DBQuery] - Inserting new order into database: user_id=67890, order_id=ORD567
2025-02-10 10:00:18 INFO [API] - Response sent: 201 Created (25ms)
2025-02-10 10:00:19 INFO [Server] - Performing scheduled maintenance check.
2025-02-10 10:00:20 WARN [Server] - High memory usage detected: 85%
2025-02-10 10:00:21 ERROR [Database] - Connection timeout when querying orders table.
2025-02-10 10:00:22 INFO [Database] - Reconnecting to database...
2025-02-10 10:00:23 INFO [Database] - Connection re-established.
`;

function WorkloadIdInfo() {
  const { workload } = Route.useLoaderData();

  return (
    <div className="mx-16 py-8 grid grid-rows-[auto_auto_1fr] gap-5 max-h-[calc(100dvh-96px)] overflow-hidden">
      <div className="flex items-center gap-2 text-[#64748B]">
        <Link to="/">Overview</Link>
        <CaretRightIcon className="w-4 h-4" />
        <span className="font-semibold text-[#2B2B2B]">{workload.name}</span>
      </div>

      <div className="flex flex-row items-start gap-4">
        <img src={workload.logoSrc} alt={`${workload.name} logo`} className="w-15.5 h-15.5" />
        <div>
          <h1 className="text-[32px] font-semibold text-[#2B2B2B]">{workload.name}</h1>
          <span className="mt-1 text-[#969FAB] leading-none">{workload.network}</span>
          <div className="flex flex-row gap-2 mt-4 flex-wrap">
            <InfoChip
              label="Status"
              value={workload.status}
              valueClassName={clsx('capitalize', {
                'text-[#69C876]': workload.status === 'connected',
                'text-[#FF7474]': workload.status === 'error',
                'text-[#2B2B2B]': workload.status === 'paused',
                'text-[#0000FF]': workload.status === 'pending',
              })}
            />
            <InfoChip label="Health" value={`${workload.uptime ?? 0}% uptime`} />
            <InfoChip label="Rewards" value={workload.rewards ?? ''} />
          </div>
        </div>
      </div>
      <div className="mt-5 max-h-full overflow-hidden">
        <Card title="Logs" className="gap-6 h-full">
          <div className="text-[13px] text-[#2B2B2B] bg-white rounded-xl p-6 min-h-0 h-full">
            <div className="overflow-y-auto whitespace-pre-wrap h-full">
              {dummyLogs}
              {dummyLogs}
              {dummyLogs}
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
