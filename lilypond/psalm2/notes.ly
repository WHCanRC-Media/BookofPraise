melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 f4 e4 d4 c4 f4 a4 g4 f4 e2 r2 \break

  % Line 2
  f2 e4 d4 a'4 g4 f4 e4 d4 f4 e2 d2 r2 \break

  % Line 3
  d2 f4 e4 d4 c4 f4 a4 g4 f4 e2 r2 \break

  % Line 4
  d2 f4 g4 a2 c2 b4 a4 g4 f4 d2 e2 d2 r2 \break

  % Line 5
  d'2 d4 d4 c2 a2 c4 b4 a4 g4 a2 d,2 r2 \break

  % Line 6
  d2 f4 g4 a2 b2 c4 d4 c4 b4 a2 r2 \break

  % Line 7
  d2 d4 d4 c2 a2 c4 b4 a4 g4 a2 d,2 r2 \break

  % Line 8
  d2 f4 g4 a2 g2 g4 f4 g2 e2 d1 \bar "|."
}
