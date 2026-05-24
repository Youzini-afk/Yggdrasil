import { motion, AnimatePresence } from "motion/react";
import { Modal } from "@/components/ui/modal";
import { ExternalWizardStep } from "./external-wizard-step";
import { SHORTCUTS } from "./install-types";
import { PlanStep } from "./plan-step";
import { ProgressStep } from "./progress-step";
import { UrlStep } from "./url-step";
import { useInstallFlow } from "./use-install-flow";

export function InstallModal({
  open,
  onClose,
  onInstalled,
}: {
  open: boolean;
  onClose: () => void;
  onInstalled?: () => void;
}) {
  const flow = useInstallFlow({ open, onClose, onInstalled });

  return (
    <Modal open={open} onOpenChange={flow.handleClose} size={flow.step === "plan" ? "lg" : "md"} contentLabel="Install project">
      <AnimatePresence mode="wait">
        {flow.step === "url" ? (
          <motion.div
            key="url"
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{ duration: 0.18 }}
          >
            <UrlStep
              url={flow.url}
              onUrlChange={flow.setUrl}
              shortcuts={SHORTCUTS}
              onSelectShortcut={(s) => flow.setUrl(s.url)}
              onContinue={flow.onContinueFromUrl}
              onCancel={flow.handleClose}
              loading={flow.isResolving}
              error={flow.resolveError}
            />
          </motion.div>
        ) : null}
        {flow.step === "plan" ? (
          <motion.div
            key="plan"
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{ duration: 0.18 }}
          >
            {flow.plan ? (
              <PlanStep
                url={flow.url}
                plan={flow.plan}
                detectedKind={flow.detectedKind}
                approvedPermissions={flow.approvedPermissions}
                onApprovalChange={flow.setApprovedPermissions}
                onBack={() => flow.setStep("url")}
                onCancel={flow.handleClose}
                onConfirm={flow.onConfirmPlan}
                installing={flow.isExecuting}
              />
            ) : null}
          </motion.div>
        ) : null}
        {flow.step === "external" ? (
          <motion.div
            key="external"
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{ duration: 0.18 }}
          >
            <ExternalWizardStep
              url={flow.url}
              plan={flow.plan}
              planError={flow.externalPlanError}
              onBack={() => flow.setStep("url")}
              onCancel={flow.handleClose}
              onContinue={flow.onContinueExternal}
            />
          </motion.div>
        ) : null}
        {flow.step === "progress" ? (
          <motion.div
            key="progress"
            initial={{ opacity: 0, x: 8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{ duration: 0.18 }}
          >
            <ProgressStep
              url={flow.url}
              plan={flow.plan}
              phases={flow.progressPhases}
              result={flow.installResult}
              error={flow.progressError}
              onCancel={flow.handleClose}
            />
          </motion.div>
        ) : null}
      </AnimatePresence>
    </Modal>
  );
}
