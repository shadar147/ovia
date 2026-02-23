"use client";

import { useState, useCallback } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useLinkIdentity } from "@/features/people/hooks/use-people";
import { useTranslation } from "@/i18n";

interface LinkIdentityDialogProps {
  personId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function LinkIdentityDialog({
  personId,
  open,
  onOpenChange,
}: LinkIdentityDialogProps) {
  const { t } = useTranslation();
  const linkMutation = useLinkIdentity(personId);
  const [identityId, setIdentityId] = useState("");

  const isValid = UUID_RE.test(identityId.trim());

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      if (!isValid) return;

      linkMutation.mutate(identityId.trim(), {
        onSuccess: () => {
          setIdentityId("");
          onOpenChange(false);
        },
      });
    },
    [identityId, isValid, linkMutation, onOpenChange],
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("person360.linkDialogTitle")}</DialogTitle>
          <DialogDescription>{t("person360.linkDialogDesc")}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">
              {t("person360.identityIdLabel")}
            </label>
            <Input
              value={identityId}
              onChange={(e) => setIdentityId(e.target.value)}
              placeholder={t("person360.identityIdPlaceholder")}
              autoFocus
            />
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              {t("people.cancel")}
            </Button>
            <Button
              type="submit"
              disabled={!isValid || linkMutation.isPending}
            >
              {linkMutation.isPending
                ? t("person360.linking")
                : t("person360.link")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
