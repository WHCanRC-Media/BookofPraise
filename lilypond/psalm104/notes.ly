melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 d2 d2 f2 d2 c2 f4 f4 g4 g4 a2 \break

  % Line 2
  r4 a4 c2 c2 d2 d2 c4 a4 c2 b2 a2 r2 \break

  % Line 3
  d,2 a'4 a4 g2 c2 b4 a4 g2 a2 f2 e2 r2 \break

  % Line 4
  d2 f2 a2 g2 a2 g4 f4 e4 d4 e2 d2 r2 \break

  % Line 5
  d'2 d4 d4 c2 g2 a4 c4 b4 b4 a2 r2 \break

  % Line 6
  d,2 d4 d4 a'2 f2 g4 a4 f4 f4 e2 r2 \break

  % Line 7
  f2 e4 d4 a'4 a4 c4 c4 d4 c4 b2 a2 r2 \break

  % Line 8
  a2 a4 a4 d,2 e2 f4 g4 f4 d4 e2 d1 \bar "|."
}
