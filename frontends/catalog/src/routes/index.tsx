import { createFileRoute } from '@tanstack/react-router';

// Components
import { Container } from '~/components/ui/Container';

// Sections
import { HeroSection } from '~/sections/home/Hero';
import { RegisterBetaSection } from '~/sections/home/RegisterBeta';
import { CuratedCatalogSection } from '~/sections/home/CuratedCatalog';
import { DeploymentSection } from '~/sections/home/Deployment';
import { HowItWorksSection } from '~/sections/home/HowItWorks';

export const Route = createFileRoute('/')({
  component: LandingPage,
});

function LandingPage() {
  return (
    <Container className="px-0 sm:px-0 py-0">
      <HeroSection />
      <CuratedCatalogSection />
      <DeploymentSection />
      <HowItWorksSection />
      <RegisterBetaSection />
    </Container>
  );
}
