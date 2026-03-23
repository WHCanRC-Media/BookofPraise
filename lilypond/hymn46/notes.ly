melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'4 e8( fis8) g4 a4 g2 fis2 g4 c4 b4 a4 b2 r4 \break

  % Line 2
  g4 e8( fis8) g4 a4 b2 a2 g4 fis4 e4 a4 d,2 r4 \break

  % Line 3
  d4 d8 e8 d4 g4 g2 fis2 fis4 g4 a4 b8( c8) b2 r4 \break

  % Line 4
  g4 a4 b4 c4 d2 g,2 c4 b8( a8) b4 a4 g2 r4 \bar "|."
}
