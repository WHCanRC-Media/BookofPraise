melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  fis4 g4 a4 b4 e,4 e4 fis2 \break

  % Line 2
  a4 b4 cis4 d4 b4 b4 a2 \break

  % Line 3
  fis4 g4 a4 b4 g4 fis4 e2 \break

  % Line 4
  a4 g8( fis8) e4 d4 e4 e4 d2 \bar "|."
}
