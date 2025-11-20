import { Section } from '~/components/Section';

export function MeasureYourReturnsSection() {
  return (
    <Section
      title="Measure your returns"
      description="SuperNode gives you clear visibility into your returns. It compares rewards to your infra spend, allowing smarter decisions about where to allocate resources."
      sideBySide
    >
      <div className="border border-zinc-200 rounded-3xl p-6 flex flex-col justify-between">
        <div className="text-zinc-800 font-semibold text-lg">Workload ROI</div>
        <img src="/images/home/graph.png" alt="Workload ROI graph" loading="lazy" />
      </div>
    </Section>
  );
}
