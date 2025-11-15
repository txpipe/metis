import { createServerFn, useServerFn } from '@tanstack/react-start';
import { setResponseStatus } from '@tanstack/react-start/server';
import { useState } from 'react';
import { twMerge } from 'tailwind-merge';
import { z } from 'zod';

// Components
import { ErrorIcon } from '~/components/icons/ErrorIcon';
import { GreenCheckIcon } from '~/components/icons/GreenCheckIcon';
import { Section } from '~/components/Section';
import { Button } from '~/components/ui/Button';

const registerToBetaSchema = z.object({
  email: z.email(),
});

const registerToBetaFn = createServerFn({
  method: 'POST',
})
  .inputValidator(registerToBetaSchema)
  .handler(async input => {
    const { email } = input.data;

    const deploymentId = process.env.BETA_SCRIPT_DEPLOYMENT_ID;
    const apiKey = process.env.BETA_API_KEY;

    if (!deploymentId || !apiKey) {
      setResponseStatus(500);
      return { success: false, error: 'Beta registration is not configured' };
    }

    const res = await fetch(`https://script.google.com/macros/s/${deploymentId}/exec`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email, apiKey }),
    });

    if (res.status !== 200) {
      setResponseStatus(400);
      return { success: false, error: 'Failed to register to beta' };
    }

    const text = await res.text();

    if (text !== 'ok') {
      setResponseStatus(400);
      return { success: false, error: 'Failed to register to beta', response: text };
    }

    setResponseStatus(200);
    return { success: true };
  });

export function RegisterBetaSection() {
  const submit = useServerFn(registerToBetaFn);
  const [isInvalid, setIsInvalid] = useState(false);
  const [submitStatus, setSubmitStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsInvalid(false);
    setSubmitStatus('idle');

    const form = e.target as HTMLFormElement;
    const formData = new FormData(form);
    const email = formData.get('email') as string;

    const parseResult = registerToBetaSchema.safeParse({ email });

    if (!parseResult.success) {
      setIsInvalid(true);
      return;
    }

    try {
      setSubmitStatus('loading');
      const submitResult = await submit({ data: { email } });
      if (!submitResult.success) {
        setIsInvalid(true);
        setSubmitStatus('error');
        return;
      }
      setSubmitStatus('success');
      form.reset();
    } catch (error) {
      // eslint-disable-next-line no-console
      console.error(error);
      setIsInvalid(true);
      setSubmitStatus('error');
    }
  };

  return (
    <Section
      id="beta"
      title="Register to beta"
      description="SuperNode is still in pre-beta — drop your email and join the first wave of collaborators as soon as we open the doors."
      center
    >
      <div>
        <form
          className={twMerge(
            'max-w-[507px] w-full focus-within:ring-1 focus-within:ring-zinc-400 border border-zinc-200 rounded-full flex items-center mx-auto px-3 py-2',
            isInvalid && 'focus-within:ring-red-400 border-red-400',
          )}
          onSubmit={onSubmit}
        >
          <input
            type="email"
            name="email"
            className="w-full text-zinc-700 placeholder:text-zinc-400 ring-0 border-0 outline-none"
            placeholder="your@emailaddress.com"
            required
            disabled={submitStatus === 'loading'}
          />
          <Button type="submit" className="min-w-min" disabled={submitStatus === 'loading'} loading={submitStatus === 'loading'}>
            {submitStatus === 'loading' ? 'Sending...' : 'Register'}
          </Button>
        </form>

        {(submitStatus === 'success' || submitStatus === 'error') && (
          <p className="mt-6 text-zinc-800 flex items-center justify-center gap-2">
            {submitStatus === 'success' && (
              <>
                <GreenCheckIcon /><span className="font-semibold">Thanks for registering!</span> We'll reach out when the beta opens.
              </>
            )}
            {submitStatus === 'error' && (
              <>
                <ErrorIcon className="text-[#ff7474]" />Registration failed. Please try again.
              </>
            )}
          </p>
        )}
      </div>
    </Section>
  );
}
