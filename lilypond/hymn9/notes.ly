melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'4 g8 g4 g8 g8 a4 a4 r2 \break

  % Line 2
  a2 a4 d4 a4 a4 a4 g4 fis2 e2 d2 r4 \break

  % Line 3
  d8 d8 g4 g4 g4 b4 b4 a4 r4 \break

  % Line 4
  d,4 a'4 a8 a4 c4 c4 b4 r2 \break

  % Line 5
  g2 b4 g4 e2 c2 d2 fis2 g1 \bar "|."
}
