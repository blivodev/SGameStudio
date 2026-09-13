import * as React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { twMerge } from 'tailwind-merge';

const button = cva(
  "inline-flex items-center justify-center rounded-md font-medium transition-colors",
  {
    variants: {
      variant: {
        default: "bg-surface text-text hover:bg-surface-alt",
        accent: "bg-accent text-white hover:bg-accent-2",
        ghost: "hover:bg-surface-alt",
      },
      size: {
        sm: "h-8 px-3 text-sm",
        md: "h-10 px-4 py-2",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "md",
    },
  }
);

export const cn = (...inputs: Parameters<typeof twMerge>): string => twMerge(...inputs);

interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof button> {}

export function Button({ className, variant, size, ...props }: ButtonProps) {
  return <button className={cn(button({ variant, size }), className)} {...props} />;
}

interface PanelProps extends React.HTMLAttributes<HTMLDivElement> {
  title?: string;
}

export function Panel({ title, className, children, ...props }: PanelProps) {
  return (
    <div className={cn("glass-panel flex flex-col", className)} {...props}>
      {title && (
        <div className="flex h-10 items-center border-b border-border px-4 font-medium">
          {title}
        </div>
      )}
      <div className="flex-1 p-4">{children}</div>
    </div>
  );
}

interface FieldProps {
  label: string;
  children: React.ReactNode;
}

export function Field({ label, children }: FieldProps) {
  return (
    <div className="flex flex-col gap-2">
      <label className="text-sm font-medium text-muted">{label}</label>
      {children}
    </div>
  );
}