# LIRS - LISP In Rust for Science

## 🧬 Lenguaje Simbólico para Diseño de Materiales

LIRS es un lenguaje de programación simbólico tipo LISP diseñado específicamente para ciencia de materiales. Combina la elegancia de LISP con dominio específico de química y física de materiales.

---

## 📚 Características Principales

### 1. **S-Expressions Químicas**
```lisp
;; Definir materiales con notación simbólica
(material :Fe 2 :O 3)  ; => "Fe2O3"
(material :Ca 1 :Ti 1 :O 3)  ; => "CaTiO3"
```

### 2. **Macros de Estructuras Cristalinas**

#### Perovskitas (ABX₃)
```lisp
(perovskite :Ca :Ti :O)  ; => "CaTiO3" (Titanato de Calcio)
(perovskite :Ba :Ti :O)  ; => "BaTiO3" (Titanato de Bario)
(perovskite :Sr :Zr :O)  ; => "SrZrO3"
```

#### Spineles (AB₂O₄)
```lisp
(spinel :Mg :Al)  ; => "MgAl2O4" (Espinela de Magnesio-Aluminio)
(spinel :Fe :Fe)  ; => "FeFe2O4" (Magnetita)
(spinel :Zn :Fe)  ; => "ZnFe2O4" (Ferrita de Zinc)
```

#### Óxidos Binarios (M₂O₃)
```lisp
(binary-oxide :Fe)  ; => "Fe2O3" (Hematita)
(binary-oxide :Al)  ; => "Al2O3" (Alúmina)
(binary-oxide :Cr)  ; => "Cr2O3"
```

#### Rock-Salt (AX)
```lisp
(rock-salt :Na :Cl)  ; => "NaCl" (Sal de mesa)
(rock-salt :Li :F)   ; => "LiF"
(rock-salt :Mg :O)   ; => "MgO" (Magnesia)
```

#### Garnets (A₃B₂C₃O₁₂)
```lisp
(garnet :Y :Al :Fe)  ; => "Y3Al2Fe3O12" (YAG)
(garnet :Ca :Fe :Si) ; => "Ca3Fe2Si3O12" (Andradita)
```

### 3. **Transformaciones Químicas**

#### Sustitución de Elementos
```lisp
;; Sustituir Ca por Sr en una perovskita
(define base (perovskite :Ca :Ti :O))
(substitute base :Ca :Sr)  ; => "SrTiO3"

;; Crear familia de materiales por sustitución
(define oxide (binary-oxide :Fe))
(substitute oxide :Fe :Co)  ; => "Co2O3"
(substitute oxide :Fe :Ni)  ; => "Ni2O3"
```

#### Combinación de Materiales
```lisp
;; Combinar dos materiales (composición compleja)
(combine "Fe2O3" "Al2O3")  ; => "Fe2Al2O6"
```

### 4. **Programación Funcional**

#### Variables y Definiciones
```lisp
(define mat1 (perovskite :Ba :Ti :O))
(define mat2 (substitute mat1 :Ba :Sr))
(define mat3 (substitute mat2 :Ti :Zr))
; mat3 => "SrZrO3"
```

#### Condicionales
```lisp
(if (> band_gap 2.0)
    (binary-oxide :Ti)   ; Semiconductor de banda ancha
    (binary-oxide :Fe))  ; Semiconductor de banda estrecha
```

#### Listas y Operaciones
```lisp
(define materials (list
    (perovskite :Ca :Ti :O)
    (perovskite :Ba :Ti :O)
    (perovskite :Sr :Ti :O)))

(car materials)  ; => "CaTiO3"
(cdr materials)  ; => lista de BaTiO3 y SrTiO3
```

### 5. **Aritmética y Lógica**
```lisp
;; Operaciones matemáticas
(+ 1 2 3 4)      ; => 10.0
(* 2 3 4)        ; => 24.0
(- 10 3)         ; => 7.0
(/ 20 4)         ; => 5.0

;; Comparaciones
(> 5 3)          ; => #t
(< 2 8)          ; => #t
(= 4 4)          ; => #t
(>= 5 5)         ; => #t
```

---

## 🤖 Integración con IA (AI-LIRS)

LIRS se integra con los motores de IA para predicción y descubrimiento:

### Predicción de Propiedades
```rust
use materials_core::lirs::ai::AILIRS;

let ai_lirs = AILIRS::with_ai(
    embedding_engine,
    ml_predictor,
    discovery_engine
);

// Predecir energía de formación
let energy = ai_lirs.predict_property(
    "formation_energy",
    "Fe2O3"
).await?;

println!("Energía de formación: {} eV", energy);
```

### Búsqueda de Similitud
```rust
// Encontrar materiales similares
let similar = ai_lirs.find_similar("CaTiO3", 10).await?;

for material in similar {
    println!("Material similar: {}", material);
}
```

### Descubrimiento Automático
```rust
// Descubrir nuevos materiales con propiedades objetivo
let candidates = ai_lirs.discover_materials(
    "band_gap",
    2.5,  // eV
    20    // máximo de candidatos
).await?;

for candidate in candidates {
    println!("Candidato: {}", candidate);
}
```

---

## 🏗️ DSL Builder - API Fluida

Para usuarios de Rust, LIRS ofrece un DSL builder:

```rust
use materials_core::lirs::dsl::{MaterialSpec, DiscoveryWorkflow};

// Especificar material declarativamente
let spec = MaterialSpec::new("perovskite")
    .with_element("Ba")
    .with_element("Ti")
    .with_element("O")
    .with_property("band_gap", 3.2)
    .with_constraint("non_toxic");

// Generar código LIRS
let lirs_code = spec.to_lirs();
// => "(perovskite :Ba :Ti :O)"

// Ejecutar
let mut lirs = LIRS::new();
let result = lirs.eval_last(&lirs_code)?;
// => "BaTiO3"
```

### Workflow de Descubrimiento
```rust
let workflow = DiscoveryWorkflow::new()
    .generate_candidates(spec)
    .substitute_element("Ba", "Sr")
    .substitute_element("Ti", "Zr")
    .combine_with("Al2O3");

let lirs_code = workflow.to_lirs();
let mut lirs = LIRS::new();
let final_material = lirs.eval_last(&lirs_code)?;
```

---

## 📖 Ejemplos Completos

### Ejemplo 1: Familia de Perovskitas
```lisp
;; Generar familia de perovskitas por sustitución sistemática
(define base-perovskite (perovskite :Ca :Ti :O))

;; Variantes del sitio A
(define sr-variant (substitute base-perovskite :Ca :Sr))
(define ba-variant (substitute base-perovskite :Ca :Ba))

;; Variantes del sitio B
(define zr-variant (substitute base-perovskite :Ti :Zr))
(define hf-variant (substitute base-perovskite :Ti :Hf))

;; Lista de todos los candidatos
(list base-perovskite sr-variant ba-variant zr-variant hf-variant)
```

### Ejemplo 2: Diseño de Materiales para Baterías
```lisp
;; Diseño de cátodos para baterías de litio
(define base-cathode (rock-salt :Li :Co))
; => "LiCoO2" (LCO tradicional)

;; Explorar sustituciones más seguras y económicas
(define nmc (substitute base-cathode :Co :Ni))  ; LiNi (NMC base)
(define lfp (substitute base-cathode :Co :Fe))  ; LiFe (LFP)
(define lmo (substitute base-cathode :Co :Mn))  ; LiMn (LMO)
```

### Ejemplo 3: Fotocatálisis
```lisp
;; Diseño de fotocatalizadores
(define tio2 (binary-oxide :Ti))  ; => "Ti2O3"

;; Explorar dopaje con metales de transición
(define fe-doped (combine tio2 "Fe2O3"))
(define co-doped (combine tio2 "Co2O3"))
(define ni-doped (combine tio2 "Ni2O3"))
```

---

## 🚀 Uso Programático

### Rust API
```rust
use materials_core::lirs::LIRS;

fn main() -> Result<(), String> {
    let mut lirs = LIRS::new();

    // Código LIRS
    let code = r#"
        (define base (perovskite :Ca :Ti :O))
        (substitute base :Ca :Sr)
    "#;

    // Evaluar
    let result = lirs.eval_last(code)?;

    // Resultado: SExpr::Atom(Atom::String("SrTiO3"))
    println!("Material: {}", result);

    Ok(())
}
```

### Macros Personalizadas
```rust
use materials_core::lirs::{LIRS, SExpr, Atom, Parser};

let mut lirs = LIRS::new();

// Definir macro personalizada: triple-oxide A B C => ABC3O9
lirs.register_macro(
    "triple-oxide",
    vec!["A".to_string(), "B".to_string(), "C".to_string()],
    Parser::new("(material A 1 B 1 C 1 :O 9)").parse().unwrap(),
);

// Usar la macro
let result = lirs.eval_last("(triple-oxide :Fe :Co :Ni)")?;
// => "FeCoNiO9"
```

---

## 🔬 Casos de Uso

### 1. **Exploración Sistemática de Composiciones**
Generar y evaluar miles de composiciones químicas programáticamente.

### 2. **Descubrimiento Guiado por IA**
Combinar programación simbólica con ML para descubrimiento acelerado.

### 3. **Diseño de Aleaciones Complejas**
Crear HEAs (High Entropy Alloys) y materiales multi-componentes.

### 4. **Prototipado Rápido**
Iterar rápidamente sobre ideas de diseño de materiales.

### 5. **Educación e Investigación**
Herramienta pedagógica para enseñar química computacional.

---

## 🎯 Próximas Características

- [ ] Operadores de simetría cristalográfica
- [ ] Cálculo simbólico de propiedades
- [ ] Generación automática de estructuras CIF
- [ ] Integración con DFT (VASP, Quantum ESPRESSO)
- [ ] Visualización 3D integrada
- [ ] REPL interactivo
- [ ] Módulos de química orgánica
- [ ] Optimización multi-objetivo

---

## 📝 Sintaxis de Referencia

### Tipos de Datos
- **Symbol**: `foo`, `my-var`, `+`
- **Integer**: `42`, `-10`
- **Float**: `3.14`, `-2.5`
- **String**: `"Fe2O3"`, `"material"`
- **Bool**: `#t` (true), `#f` (false)
- **Element**: `:Fe`, `:O`, `:Ca` (prefijo `:`)
- **Nil**: `nil`

### Funciones Built-in
- **Aritmética**: `+`, `-`, `*`, `/`
- **Comparación**: `=`, `>`, `<`, `>=`, `<=`
- **Listas**: `list`, `car`, `cdr`
- **Control**: `if`, `define`
- **Química**: `material`, `substitute`, `combine`
- **Macros**: `perovskite`, `spinel`, `binary-oxide`, `rock-salt`, `garnet`

---

## 🌟 Conclusión

**LIRS** representa un paradigma nuevo en diseño computacional de materiales:
- **Expresivo**: Sintaxis limpia y potente
- **Extensible**: Macros y funciones definidas por usuario
- **Inteligente**: Integración nativa con IA
- **Científico**: Diseñado para química y física de materiales

**"Concebir lo inconcebible en el diseño de materiales"** 🚀

---

© 2025 Materials-Simulato-R - LIRS v1.0.0
