import { createFileRoute } from '@tanstack/react-router';

// Components
import { Container } from '~/components/ui/Container';

// Sections
import { HeroSection } from '~/sections/home/hero';
import { RegisterBetaSection } from '~/sections/home/register-beta';

export const Route = createFileRoute('/')({
  component: LandingPage,
});

function LandingPage() {
  return (
    <Container className="px-0 py-0">
      <HeroSection />
      <RegisterBetaSection />
    </Container>
  );
}
