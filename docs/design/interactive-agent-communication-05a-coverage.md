# Interactive agent communication: step 5, coverage tables (sub-document of the evidence specification)

**Task:** T-3354 · **Arc:** arc-011 · **Chain:** arc-011-design, step 5 · **Role:** evidence specialist
**Status:** DRAFT v0.1, part of `docs/design/interactive-agent-communication-05-evidence.md` ("EV5"), reviewed with it.
Card 4.2: a coverage table R-n / SI-n / FAILS-CLOSED condition → E-n, with gaps marked.

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-11-09 | Generated from EV5 v0.1: R-1..R-82 and R-67.a1, SI-1..SI-31, FX-1..FX-8, "impossible" requirements, adversarial coverage of every SI-n, AFC-1..AFC-34, the ARC4 15 measured properties, counts by category and level | T-3354 |

## 1 How these tables are made

1.1 Generated, not hand-written: `python3 .context/working/t3354/gen_coverage.py` (a working tool of this step, kept
beside the render script as step 4 did, not product code), sha256
`bb2d4a7e885e1b149fc11735f27e7e220ce5ec1a434ceab1a623c1a40d9c4766`, run on EV5 with sha256
`29be80ae3ad8d32b71ef53caa00f9fdc6d03426c901215eaed85c019f889526e`. It reads every spec header (`Links:`,
`Category:`, `Level:`), every row of EV5 section 10 and the AFC table of EV5 section 13, and exits 0 only when every
item has a spec or a declared gap (1 when an item is uncovered and undeclared, 2 on a parse problem). Exit at
generation: 0.
1.2 "Covered" means at least one spec names the item in its links. It does not mean the spec has run: EV5 11.4 lists
what cannot run until its estate exists.
1.3 Declared gaps carry their reason in the Status column. The pseudo-code's FAILS CLOSED WHEN conditions are not in
these tables because they do not exist yet (EV5 section 14); the next iteration adds table 2.9 "FCW-n → E-n".

## 2 Tables

### 2.1 Requirements R-1 to R-82 (and R-67.a1)

| Item | Specs | Status |
|---|---|---|
| R-1 | E-77, E-109, E-111, E-146 | covered |
| R-2 | E-75, E-77, E-93, E-109, E-123, E-146 | covered |
| R-3 | E-1, E-75, E-77, E-102 | covered |
| R-4 | E-68 | covered |
| R-5 | E-93, E-109 | covered |
| R-6 | E-19, E-55, E-62, E-63, E-64, E-102 | covered |
| R-7 | E-60, E-102, E-110 | covered |
| R-8 | E-65, E-80 | covered |
| R-9 | E-112 | covered |
| R-10 | E-110 | covered |
| R-11 | E-44, E-45, E-46 | covered |
| R-12 | E-110 | covered |
| R-13 | E-102, E-109 | covered |
| R-14 | E-75, E-93 | covered |
| R-15 | E-44, E-45, E-75, E-81, E-90, E-92, E-93, E-122 | covered |
| R-16 | E-92, E-93, E-122 | covered |
| R-17 | E-102 | covered |
| R-18 | E-89 | covered |
| R-19 | E-1, E-7, E-9, E-11, E-102 | covered |
| R-20 | E-1, E-2, E-5, E-13, E-15, E-17, E-18, E-121, E-137 | covered |
| R-21 | E-16, E-18, E-84 | covered |
| R-22 | E-2, E-8, E-121 | covered |
| R-23 | E-3, E-27, E-125 | covered |
| R-24 | E-4, E-6, E-7, E-74, E-75, E-77, E-84, E-93, E-97, E-98, E-146 | covered |
| R-25 | E-5, E-89 | covered |
| R-26 | E-75 | covered |
| R-27 | E-75 | covered |
| R-28 | E-76, E-89, E-123 | covered |
| R-29 | E-10, E-19, E-28, E-55, E-56, E-57, E-63, E-79, E-147, E-149 | covered |
| R-30 | E-54, E-89 | covered |
| R-31 | E-89 | covered |
| R-32 | E-110 | covered |
| R-33 | E-36 | covered |
| R-34 | E-99 | covered |
| R-35 | E-67 | covered |
| R-36 | — | GAP (declared): deferred to PH-6 (daily digest and reflection; TRACE: 'PH-6 digests'); no spec until its sub-design |
| R-37 | E-149 | covered |
| R-38 | — | GAP (declared): deferred by ruling (observability database, hub steward); nothing to evidence |
| R-39 | E-80, E-100 | covered |
| R-40 | — | GAP (declared): merged into R-28 (OD-13); evidenced by E-76 |
| R-41 | — | GAP (declared): deferred to the planner (typed assignment/result messages); nothing to evidence yet |
| R-42 | E-4, E-45, E-74, E-77, E-93, E-100, E-148 | covered |
| R-43 | — | GAP (declared): dropped (same sentence as R-9) |
| R-44 | E-5, E-7, E-9, E-11, E-57, E-60, E-70, E-75, E-76, E-77, E-123, E-138, E-147 | covered |
| R-45 | E-60, E-75, E-88, E-90, E-93, E-109, E-117 | covered |
| R-46 | E-39, E-110, E-117 | covered |
| R-47 | E-16, E-29, E-71, E-84, E-85 | covered |
| R-48 | E-102 | covered |
| R-49 | E-29, E-63, E-66, E-81, E-82, E-83 | covered |
| R-50 | E-84, E-95, E-103 | covered |
| R-51 | E-6, E-32, E-33, E-35, E-84, E-93, E-94, E-98 | covered |
| R-52 | E-89, E-124 | covered |
| R-53 | E-28, E-55, E-56, E-57, E-61 | covered |
| R-54 | E-36, E-38, E-63, E-79, E-145 | covered |
| R-55 | E-109 | covered |
| R-56 | E-50, E-51, E-52, E-53, E-54 | covered |
| R-57 | E-59 | covered |
| R-58 | E-89, E-123, E-130 | covered |
| R-59 | E-86, E-87 | covered |
| R-60 | E-66, E-108 | covered |
| R-61 | E-36, E-37, E-126 | covered |
| R-62 | E-32, E-34, E-70, E-99 | covered |
| R-63 | E-12, E-41, E-106, E-124 | covered |
| R-64 | E-106 | covered |
| R-65 | E-112 | covered |
| R-66 | E-75, E-78, E-141 | covered |
| R-67 | E-20, E-21, E-22, E-23, E-24, E-25, E-26, E-27, E-28, E-29, E-30, E-31, E-85, E-107, E-139, E-142 | covered |
| R-68 | E-20, E-24, E-25, E-26, E-30 | covered |
| R-69 | E-62, E-75, E-78, E-79, E-93, E-135, E-151 | covered |
| R-70 | E-55, E-63, E-150 | covered |
| R-71 | E-118 | covered |
| R-72 | E-103, E-104, E-116 | covered |
| R-73 | E-106 | covered |
| R-74 | E-111 | covered |
| R-75 | E-37, E-69 | covered |
| R-76 | E-104, E-105, E-115 | covered |
| R-77 | E-108 | covered |
| R-78 | E-106, E-107 | covered |
| R-79 | E-71, E-72, E-74, E-84 | covered |
| R-80 | E-44, E-47, E-48, E-49, E-119, E-120, E-144 | covered |
| R-81 | E-2, E-13, E-15 | covered |
| R-82 | E-58, E-113, E-114, E-115 | covered |
| R-67.a1 | E-32, E-33, E-34, E-35, E-143 | covered |

### 2.2 Security invariants SI-1 to SI-31

| Item | Specs | Status |
|---|---|---|
| SI-1 | E-3, E-125 | covered |
| SI-2 | E-1, E-2, E-8, E-13, E-14, E-15, E-18, E-121 | covered |
| SI-3 | E-34, E-69 | covered |
| SI-4 | E-94 | covered |
| SI-5 | E-86, E-87, E-128 | covered |
| SI-6 | E-87, E-88 | covered |
| SI-7 | E-95 | covered |
| SI-8 | E-103 | covered |
| SI-9 | E-59, E-110 | covered |
| SI-10 | E-110 | covered |
| SI-11 | E-37, E-39, E-40, E-41, E-42, E-43, E-127 | covered |
| SI-12 | E-108, E-116 | covered |
| SI-13 | E-104 | covered |
| SI-14 | E-58, E-96, E-114 | covered |
| SI-15 | E-65, E-81, E-90, E-92, E-117, E-122 | covered |
| SI-16 | E-63, E-66, E-83, E-133 | covered |
| SI-17 | E-4, E-97 | covered |
| SI-18 | E-41, E-89, E-124 | covered |
| SI-19 | E-50, E-51, E-53, E-129 | covered |
| SI-20 | E-20, E-23, E-24, E-25, E-30, E-107, E-142 | covered |
| SI-21 | E-106 | covered |
| SI-22 | E-91, E-117, E-132 | covered |
| SI-23 | E-2, E-8, E-12, E-113 | covered |
| SI-24 | E-43, E-67, E-113, E-115 | covered |
| SI-25 | E-37, E-101 | covered |
| SI-26 | E-34, E-70, E-99 | covered |
| SI-27 | E-80, E-100 | covered |
| SI-28 | E-44, E-46 | covered |
| SI-29 | E-111 | covered |
| SI-30 | E-4, E-6, E-35, E-98 | covered |
| SI-31 | E-105 | covered |

### 2.3 Floor items FX-1 to FX-8

| Item | Specs | Status |
|---|---|---|
| FX-1 | E-71, E-72, E-73, E-74, E-140 | covered |
| FX-2 | E-36, E-38, E-63, E-126, E-145 | covered |
| FX-3 | E-112 | covered |
| FX-4 | E-68 | covered |
| FX-5 | E-44, E-47, E-48, E-49, E-120, E-131 | covered |
| FX-6 | E-67, E-133 | covered |
| FX-7 | E-4, E-77, E-93, E-100, E-148 | covered |
| FX-8 | E-10, E-19, E-28, E-55, E-56, E-57, E-63, E-79, E-134 | covered |

### 2.4 'Impossible' requirements (REQ field .f): adversarial spec required

| Item | Adversarial specs | Status |
|---|---|---|
| R-2 | E-123 | covered |
| R-15 | E-122 | covered |
| R-16 | E-122 | covered |
| R-19 | E-9 | covered |
| R-20 | E-121 | covered |
| R-22 | E-121 | covered |
| R-23 | E-3, E-27 | covered |
| R-24 | E-4, E-97 | covered |
| R-28 | E-123 | covered |
| R-29 | E-56, E-79 | covered |
| R-34 | E-99 | covered |
| R-42 | E-4, E-100 | covered |
| R-72 | E-103, E-104, E-116 | covered |
| R-82 | E-58, E-113, E-114, E-115 | covered |

### 2.5 Every SI-n has an adversarial spec

| Item | Adversarial specs | Status |
|---|---|---|
| SI-1 | E-3 | covered |
| SI-2 | E-14, E-121 | covered |
| SI-3 | E-34, E-69 | covered |
| SI-4 | E-94 | covered |
| SI-5 | E-86 | covered |
| SI-6 | E-88 | covered |
| SI-7 | E-95 | covered |
| SI-8 | E-103 | covered |
| SI-9 | E-110 | covered |
| SI-10 | E-110 | covered |
| SI-11 | E-37, E-39, E-40, E-42, E-43 | covered |
| SI-12 | E-108, E-116 | covered |
| SI-13 | E-104 | covered |
| SI-14 | E-58, E-96, E-114 | covered |
| SI-15 | E-122 | covered |
| SI-16 | E-66 | covered |
| SI-17 | E-4, E-97 | covered |
| SI-18 | E-124 | covered |
| SI-19 | E-51, E-53 | covered |
| SI-20 | E-20, E-24, E-107 | covered |
| SI-21 | E-106 | covered |
| SI-22 | E-91 | covered |
| SI-23 | E-113 | covered |
| SI-24 | E-43, E-67, E-113, E-115 | covered |
| SI-25 | E-37, E-101 | covered |
| SI-26 | E-34, E-99 | covered |
| SI-27 | E-100 | covered |
| SI-28 | E-46 | covered |
| SI-29 | E-111 | covered |
| SI-30 | E-4 | covered |
| SI-31 | E-105 | covered |

### 2.6 Architecture fail-closed conditions AFC-n (section 13)

| Item | Specs | Status |
|---|---|---|
| AFC-1 | E-1 | covered |
| AFC-2 | E-2 | covered |
| AFC-3 | E-3 | covered |
| AFC-4 | E-4, E-97 | covered |
| AFC-5 | E-5, E-52 | covered |
| AFC-6 | E-6, E-98 | covered |
| AFC-7 | E-7, E-11 | covered |
| AFC-8 | E-8 | covered |
| AFC-9 | E-9, E-17 | covered |
| AFC-10 | E-10 | covered |
| AFC-11 | E-11 | covered |
| AFC-12 | E-12 | covered |
| AFC-13 | E-15 | covered |
| AFC-14 | E-16, E-84 | covered |
| AFC-15 | E-18 | covered |
| AFC-16 | E-20, E-21, E-23, E-24, E-25, E-26, E-27 | covered |
| AFC-17 | E-22 | covered |
| AFC-18 | E-28, E-29 | covered |
| AFC-19 | E-32, E-33, E-34 | covered |
| AFC-20 | E-36, E-37 | covered |
| AFC-21 | E-38 | covered |
| AFC-22 | E-39, E-40, E-41, E-42 | covered |
| AFC-23 | E-44, E-45, E-46, E-47 | covered |
| AFC-24 | E-48 | covered |
| AFC-25 | E-50, E-51 | covered |
| AFC-26 | E-53 | covered |
| AFC-27 | E-19, E-55 | covered |
| AFC-28 | E-90 | covered |
| AFC-29 | E-49 | covered |
| AFC-30 | E-82 | covered |
| AFC-31 | E-72 | covered |
| AFC-32 | E-67 | covered |
| AFC-33 | E-69 | covered |
| AFC-34 | E-70 | covered |

### 2.7 Measured properties of ARC4 section 15 (invariant checks, section 10)

| Item | Specs | Status |
|---|---|---|
| K-9a | E-126 | covered |
| K-9b | E-127 | covered |
| K-10 | E-128 | covered |
| K-11 | E-129 | covered |
| K-12 | E-130 | covered |
| K-13 | E-131 | covered |
| K-14 | E-132 | covered |
| K-15 | E-133 | covered |
| K-16 | E-134 | covered |
| K-17 | E-135 | covered |
| K-19 | E-125, E-136 | covered |
| K-32 | E-137 | covered |
| K-33 | E-138 | covered |
| K-34 | E-139 | covered |
| K-20 | E-140 | covered |
| K-21 | E-141 | covered |
| K-22 | E-142 | covered |
| K-23 | E-143 | covered |
| K-24 | E-144 | covered |
| K-25 | E-145 | covered |

### 2.8 Specs by category and level

| Category | Specs |
|---|---|
| adversarial | 58 |
| core function | 50 |
| edge case | 10 |
| error handling | 15 |
| integration | 15 |
| user workflow | 4 |

| Level (a spec may name two) | Specs |
|---|---|
| L1 | 4 |
| L2 | 59 |
| L3 | 5 |
| L4 | 57 |
| L5 | 11 |
| L6 | 9 |
| L7 | 33 |

Total specs: 152. Undeclared gaps: 0.
