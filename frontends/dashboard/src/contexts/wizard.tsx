import { createContext, useContext, useState } from 'react';
import type { ReactNode, Dispatch, SetStateAction } from 'react';

export enum StepStatus {
  UPCOMING,
  CURRENT,
  PROCESSING,
  ERROR,
  COMPLETED,
}

type WizardContextType = {
  badgeStatus: string;
  setBadgeStatus: Dispatch<SetStateAction<string>>;
  stepStatus: StepStatus;
  setStepStatus: Dispatch<SetStateAction<StepStatus>>;
};

const WizardContext = createContext<WizardContextType | undefined>(undefined);

export function WizardProvider({ children, step }: { children: ReactNode; step: number; }) {
  const [badgeStatus, setBadgeStatus] = useState(step === 1 ? 'Ready to start' : 'In-progress');
  const [stepStatus, setStepStatus] = useState<StepStatus>(
    step === 3 ? StepStatus.PROCESSING : StepStatus.CURRENT,
  );

  return (
    <WizardContext.Provider
      value={{
        badgeStatus,
        setBadgeStatus,
        stepStatus,
        setStepStatus,
      }}
    >
      {children}
    </WizardContext.Provider>
  );
}

export function useWizard() {
  const context = useContext(WizardContext);
  if (!context) throw new Error('useWizard must be used within WizardProvider');
  return context;
}
