melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 b2 a2 fis2 a2 g4 fis4 g2 e2 d2 \break

  % Line 2
  r4 d4 fis2 g2 a2 a2 b4 a4 fis4 g4 a2 r2 \break

  % Line 3
  a2 a4 g4 fis2 fis2 fis4 e4 fis4 a4 g2 fis2 r2 \break

  % Line 4
  fis2 a4 g4 fis2 b2 b4 b4 a4 g4 fis2 e2 \break

  % Line 5
  r4 fis4 a2 a2 b2 d2 cis4 b4 a2 b2 a2 r2 \break

  % Line 6
  a2 b2 a2 fis2 a2 g4 fis4 g2 e2 d1 \bar "|."
}
