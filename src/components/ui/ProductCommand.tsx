import { useState } from "react"
import { Dialog as DialogPrimitive } from "radix-ui"
import { HugeiconsIcon } from "@hugeicons/react"
import { Search01Icon, Tick01Icon, Add01Icon } from "@hugeicons/core-free-icons"
import type { Product } from "@/types/product"
import type { UnitOfMeasure } from "@/types/units"
import { scoreProduct } from "@/lib/productSearch"
import { cn } from "@/lib/utils"
import {
  Command,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })

export type NewProductDraft = {
  name: string
  unit: string
  description: string
  sellable: boolean
}

type Props = {
  open: boolean
  onOpenChange: (open: boolean) => void
  products: Product[]
  onSelect: (product: Product) => void
  selectedIds?: number[]
  units?: UnitOfMeasure[]
  onCreateNew?: (draft: NewProductDraft) => void
}

export function ProductCommand({
  open,
  onOpenChange,
  products,
  onSelect,
  selectedIds = [],
  units = [],
  onCreateNew,
}: Props) {
  const [query, setQuery] = useState("")
  const [miniForm, setMiniForm] = useState<NewProductDraft | null>(null)

  const activeUnits = units.filter((u) => u.active)

  function handleOpenChange(v: boolean) {
    if (!v) {
      setMiniForm(null)
      setQuery("")
    }
    onOpenChange(v)
  }

  function openMiniForm() {
    setMiniForm({
      name: query,
      unit: activeUnits[0]?.name ?? "",
      description: "",
      sellable: false,
    })
  }

  function submitMiniForm() {
    if (!miniForm || !miniForm.name.trim() || !miniForm.unit) return
    onCreateNew?.(miniForm)
    handleOpenChange(false)
  }

  return (
    <DialogPrimitive.Root open={open} onOpenChange={handleOpenChange}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="fixed inset-0 z-50 bg-black/60 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0" />
        <DialogPrimitive.Content
          className={cn(
            "fixed left-1/2 top-[25%] z-50 w-full max-w-lg -translate-x-1/2 overflow-hidden rounded-xl border border-teller-border bg-teller-card shadow-2xl outline-none",
            "data-[state=open]:animate-in data-[state=closed]:animate-out",
            "data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
            "data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
          )}
        >
          <DialogPrimitive.Title className="sr-only">
            {miniForm ? "Criar produto" : "Buscar produto"}
          </DialogPrimitive.Title>

          {miniForm ? (
            <div className="p-4 flex flex-col gap-3">
              <p className="text-sm font-medium text-teller-text">Novo produto</p>

              <div className="flex flex-col gap-1.5">
                <Label htmlFor="np-name">Nome</Label>
                <Input
                  id="np-name"
                  autoFocus
                  value={miniForm.name}
                  onChange={(e) => setMiniForm({ ...miniForm, name: e.target.value })}
                />
              </div>

              <div className="flex flex-col gap-1.5">
                <Label htmlFor="np-unit">Unidade</Label>
                <select
                  id="np-unit"
                  value={miniForm.unit}
                  onChange={(e) => setMiniForm({ ...miniForm, unit: e.target.value })}
                  className="flex h-10 w-full rounded-md border border-teller-border bg-teller-card px-3 py-2 text-sm text-teller-text focus:outline-none focus:ring-2 focus:ring-teller-accent"
                >
                  {activeUnits.map((u) => (
                    <option key={u.id} value={u.name}>
                      {u.name} ({u.abbreviation})
                    </option>
                  ))}
                  {activeUnits.length === 0 && (
                    <option value="">Nenhuma unidade ativa</option>
                  )}
                </select>
              </div>

              <div className="flex flex-col gap-1.5">
                <Label htmlFor="np-desc">Descricao (opcional)</Label>
                <Input
                  id="np-desc"
                  value={miniForm.description}
                  onChange={(e) => setMiniForm({ ...miniForm, description: e.target.value })}
                />
              </div>

              <div className="flex items-center justify-between">
                <Label htmlFor="np-sellable">Vendavel</Label>
                <Switch
                  id="np-sellable"
                  checked={miniForm.sellable}
                  onCheckedChange={(checked) => setMiniForm({ ...miniForm, sellable: checked })}
                />
              </div>

              <div className="flex gap-2 justify-end pt-1">
                <Button variant="ghost" size="sm" onClick={() => setMiniForm(null)}>
                  Voltar
                </Button>
                <Button
                  size="sm"
                  onClick={submitMiniForm}
                  disabled={!miniForm.name.trim() || !miniForm.unit}
                >
                  Confirmar
                </Button>
              </div>
            </div>
          ) : (
            <Command
              filter={(value, search) => {
                if (value === "__create_new__") {
                  return onCreateNew && search.trim() ? 1 : 0
                }
                const product = products.find((p) => String(p.id) === value)
                if (!product) return 0
                return scoreProduct(product, search)
              }}
            >
              <div className="flex items-center gap-3 px-4 border-b border-teller-border">
                <HugeiconsIcon icon={Search01Icon} size={16} className="text-teller-muted shrink-0" />
                <CommandInput
                  placeholder="Buscar produto..."
                  className="h-12"
                  onValueChange={setQuery}
                />
              </div>
              <CommandList className="max-h-72 p-1">
                <CommandGroup>
                  {onCreateNew && (
                    <CommandItem
                      value="__create_new__"
                      onSelect={openMiniForm}
                      className="text-teller-accent gap-2"
                    >
                      <HugeiconsIcon icon={Add01Icon} size={14} className="shrink-0" />
                      <span>
                        Criar novo:{" "}
                        <span className="font-medium">{query || "..."}</span>
                      </span>
                    </CommandItem>
                  )}
                  {products.map((product) => {
                    const isSelected = selectedIds.includes(product.id)
                    return (
                      <CommandItem
                        key={product.id}
                        value={String(product.id)}
                        onSelect={() => {
                          onSelect(product)
                          onOpenChange(false)
                        }}
                        className="flex items-center justify-between gap-2 rounded-lg px-3 py-2"
                      >
                        <span className="truncate">{product.name}</span>
                        <div className="flex items-center gap-2 shrink-0">
                          {isSelected && (
                            <HugeiconsIcon icon={Tick01Icon} size={14} className="text-teller-accent" />
                          )}
                          <span className="text-teller-muted text-xs tabular-nums">
                            {brl.format(product.final_price)}
                          </span>
                        </div>
                      </CommandItem>
                    )
                  })}
                </CommandGroup>
              </CommandList>
            </Command>
          )}
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  )
}
