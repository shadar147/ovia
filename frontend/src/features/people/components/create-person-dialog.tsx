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
import { useCreatePerson } from "@/features/people/hooks/use-people";
import { useTranslation } from "@/i18n";

interface CreatePersonDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CreatePersonDialog({
  open,
  onOpenChange,
}: CreatePersonDialogProps) {
  const { t } = useTranslation();
  const createMutation = useCreatePerson();

  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [team, setTeam] = useState("");
  const [role, setRole] = useState("");

  const reset = useCallback(() => {
    setDisplayName("");
    setEmail("");
    setTeam("");
    setRole("");
  }, []);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      if (!displayName.trim()) return;

      createMutation.mutate(
        {
          display_name: displayName.trim(),
          primary_email: email.trim() || undefined,
          team: team.trim() || undefined,
          role: role.trim() || undefined,
        },
        {
          onSuccess: () => {
            reset();
            onOpenChange(false);
          },
        },
      );
    },
    [displayName, email, team, role, createMutation, reset, onOpenChange],
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("people.createTitle")}</DialogTitle>
          <DialogDescription>{t("people.createDesc")}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">{t("people.fieldName")} *</label>
            <Input
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder={t("people.fieldName")}
              required
              autoFocus
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">{t("people.fieldEmail")}</label>
            <Input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder={t("people.fieldEmail")}
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">{t("people.fieldTeam")}</label>
            <Input
              value={team}
              onChange={(e) => setTeam(e.target.value)}
              placeholder={t("people.fieldTeam")}
            />
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">{t("people.fieldRole")}</label>
            <Input
              value={role}
              onChange={(e) => setRole(e.target.value)}
              placeholder={t("people.fieldRole")}
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
              disabled={!displayName.trim() || createMutation.isPending}
            >
              {createMutation.isPending
                ? t("people.creating")
                : t("people.create")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
