import { createFileRoute } from '@tanstack/react-router';

// Components
import { Container } from '~/components/ui/Container';

// Sections
import { HeroSection } from '~/sections/home/Hero';
import { RegisterBetaSection } from '~/sections/home/RegisterBeta';
import { CuratedCatalogSection } from '~/sections/home/CuratedCatalog';
import { DeploymentSection } from '~/sections/home/Deployment';
import { HowItWorksSection } from '~/sections/home/HowItWorks';
import { CommunitySection } from '~/sections/home/Community';
import { UnifiedOperationalLayerSection } from '~/sections/home/UnifiedOperationalLayer';
import { MeasureYourReturnsSection } from '~/sections/home/MeasureReturns';

export const Route = createFileRoute('/')({
  component: LandingPage,
});

function LandingPage() {
  return (
    <Container className="px-0 sm:px-0 py-0">
      <HeroSection />
      <CuratedCatalogSection />
      <UnifiedOperationalLayerSection />
      <DeploymentSection />
      <MeasureYourReturnsSection />
      <HowItWorksSection />
      <RegisterBetaSection />
      <CommunitySection />
    </Container>
  );
}
