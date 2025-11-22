# 🧬 Experimento de Compresión de Embeddings ML

**Autor**: Francisco Molina Burgos (ORCID: 0009-0008-6093-8267)
**Fecha**: 2025-11-21
**Status**: ✅ COMPLETADO - Fase 1-3b

---

## 🎯 Objetivo

Investigar y desarrollar métodos de compresión extrema para vectores de embeddings de ML (768D), con enfoque en:

1. Validación de Delta Encoding
2. Análisis de atractores caóticos
3. Desarrollo de compresión basada en estructura de baja dimensión

---

## 📊 Resultados Principales

### Compresión Lograda

| Método | Ratio Promedio | Accuracy Loss |
|--------|----------------|---------------|
| **Attractor(PCA-10)** | **223.94x** | 86.7% |
| Int8+GZIP | 9.06x | 22.5% |
| Delta+ANS | 4.71x | 15.5% |
| Polar Delta | 2.65x | 2.1% |
| Delta+GZIP | 1.10x | 0% |

### Hallazgo Crítico: Atractor Caótico

✅ **CONFIRMADO** en dataset "Clustered Topics":
- **Dimensión de correlación D₂ = 0.53**
- **Exponente de Lyapunov λ₁ = 0.645** (caótico)
- **Potencial teórico: 1,445x compresión**

---

## 🚀 Inicio Rápido

### Compilar

```bash
cargo build --release
```

### Ejecutar Experimento Completo (9 métodos)

```bash
cargo run --release --bin compression-experiment
```

### Análisis de Atractores

```bash
cargo run --release --bin analyze_attractor
```

### Diagnóstico de Deltas

```bash
cargo run --release --bin analyze_deltas
```

---

## 📞 Contacto

**Francisco Molina Burgos**
- ORCID: [0009-0008-6093-8267](https://orcid.org/0009-0008-6093-8267)
- Email: pako.molina@gmail.com
- GitHub: [@Yatrogenesis](https://github.com/Yatrogenesis)

---

## 📄 Documentación Completa

Ver [`REPORTE_FINAL_COMPLETO.md`](./REPORTE_FINAL_COMPLETO.md) para análisis científico detallado.

---

**Última actualización**: 2025-11-21
**Status**: ✅ FASE 3b COMPLETADA
