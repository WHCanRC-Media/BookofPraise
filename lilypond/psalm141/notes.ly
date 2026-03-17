melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  fis2 e4 d2 fis4 a2 g4 fis4 b2 a2 r2 \break

  % Line 2
  a2 b4 b4 a2 fis4 g2 e4 fis2 r2 \break

  % Line 3
  a2 g4 e4 fis2 g4 fis2 e4 d2 r2 \break

  % Line 4
  d2 fis4 g4 a2 a2 fis4 a4 g2 fis1 \bar "|."
}
