melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'4 g4 a8 g4 e4 f8 g4 f4 e8 d4 e4 c4 \break

  % Line 2
  g'4 g4 a8 b4 c4 d8 b4 a4 g8 a4 g2 \break

  % Line 3
  g4 g4 a8 b4 c4 g4 g4 g4( a8) f8 g8 e4 c4 \break

  % Line 4
  f4 f4 g8 a8 f8 g4( f8) e4 f4 d4 c8 c2 \bar "|."
}
