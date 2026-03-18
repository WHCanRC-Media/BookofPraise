melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 a'2 a2 b4 cis4 d2 d4 cis4 b2 a2 \break

  % Line 2
  fis2 b4 b4 a4 gis4 a2 r2 \break

  % Line 3
  fis2 g2 a2 fis4 d4 g4 fis4 e2 d2 \break

  % Line 4
  a'2 g4 fis4 d4 e4 fis2 r2 \break

  % Line 5
  fis2 d4 e4 fis4 a4 g2 fis2 e4 \break

  % Line 6
  a4 g4 fis4 d2 e2 d1 \bar "|."
}
