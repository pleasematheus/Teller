import { Dialog as DialogPrimitive } from "radix-ui"
import { HugeiconsIcon } from "@hugeicons/react"
import { Search01Icon, Tick01Icon } from "@hugeicons/core-free-icons"
import type { Product } from "@/types/product"
import { scoreProduct } from "@/lib/productSearch"
import { cn } from "@/lib/utils"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command"

const brl = new Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })

type Props = {
  open: boolean
  onOpenChange: (open: boolean) => void
  products: Product[]
  onSelect: (product: Product) => void
  selectedIds?: number[]
}

export function ProductCommand({ open, onOpenChange, products, onSelect, selectedIds = [] }: Props) {
  return (
    <DialogPrimitive.Root open={open} onOpenChange={onOpenChange}>
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
          <DialogPrimitive.Title className="sr-only">Buscar produto</DialogPrimitive.Title>
          <Command
            filter={(value, search) => {
              const product = products.find((p) => String(p.id) === value)
              if (!product) return 0
              return scoreProduct(product, search)
            }}
          >
            <div className="flex items-center gap-3 px-4 border-b border-teller-border">
              <HugeiconsIcon icon={Search01Icon} size={16} className="text-teller-muted shrink-0" />
              <CommandInput placeholder="Buscar produto..." className="h-12" />
            </div>
            <CommandList className="max-h-72 p-1">
              <CommandEmpty>Nenhum produto encontrado.</CommandEmpty>
              <CommandGroup>
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
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </DialogPrimitive.Root>
  )
}
