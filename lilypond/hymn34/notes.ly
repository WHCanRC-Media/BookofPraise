melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d4 d4 d4 a'2 b4 c2 a4 g2 \break

  % Line 2
  a4 b2 c4 d2 a4 c4 a2 \break

  % Line 3
  a4 c2 a4 g2 d4 f2 d4 c2 \break

  % Line 4
  c4 f2 g4 a2 g4 f2 c'4 a2 g4 f4( g4) e4 d2 \bar "|."
}
