melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d4 g8 g8 a4 a4 b8 a8 g4 \break

  % Line 2
  a4 b8 b8 c4 b4 a2 \break

  % Line 3
  d4 d8 b8 b4 g4 g8 e8 e4 \break

  % Line 4
  g8( e8) d8 g8 g4 a4 g2 \bar "|."
}
