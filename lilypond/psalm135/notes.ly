melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d'2 b2 a4 a4 b4 cis4 d2 r2 \break

  % Line 2
  a2 b2 a4 g4 fis2 e2 d2 r2 \break

  % Line 3
  d2 fis2 e4 d4 fis4 g4 a2 r2 \break

  % Line 4
  fis2 a2 b4 cis4 d4 b4 a2 r2 \break

  % Line 5
  a2 g2 fis4 a4 g4 fis4 e2 r2 \break

  % Line 6
  e2 fis2 a4 g4 fis2 e2 d1 \bar "|."
}
