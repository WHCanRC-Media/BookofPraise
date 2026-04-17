melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'4 a4 a4 g4 bes4 a2 g4 \break

  % Line 2
  f4 e4 d4 e4 e4 d2 r4 \break

  % Line 3
  a'4 a4 a4 g4 bes4 a2 g4 \break

  % Line 4
  f4 e4 d4 e4 e4 d2 r4 \break

  % Line 5
  d4 f4 f4 g4 g4 a2 f4 \break

  % Line 6
  a4 a4 a4 g4 g4 f2 r4 \break

  % Line 7
  c'4 c4 c4 a8( g8) f4 e2 d4 \break

  % Line 8
  f4 e4 d4 e4 e4 d1 \bar "|."
}
